use macroquad::math::{Rect, Vec2, vec2};
use mlua::Table;

use crate::pathfinder;

const BOUNDARY_EPS: f32 = 0.01;
const MIN_MITER_DENOMINATOR: f32 = 0.1;
/// Frames a second for an animated prop that does not say.
pub const DEFAULT_FPS: f32 = 8.0;

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
}

/// An image placed in the room; characters with a smaller y than the depth are drawn in front of it.
#[derive(Clone, Debug)]
pub struct Prop {
    /// Scripts hide a prop (and the hotspot with the same id) by this name.
    pub id: Option<String>,
    pub image: String,
    pub x: f32,
    pub y: f32,
    pub depth: f32,
    /// An animated prop's image is a strip of this many equal frames side by side (1 if it is not animated).
    pub frames: usize,
    /// How fast `animate` plays them, when the script does not say.
    pub fps: f32,
    /// Loops this many leading frames for ever while no script is animating the prop (0: stays on frame 0).
    pub idle: usize,
    /// Drawn above the darkness of an unlit room, so it can still be seen (a light switch).
    pub glow: bool,
    /// Image pixels per game unit: 1 for art drawn at the game's own resolution, 4 for art drawn at
    /// four times it (drawn smaller, so it looks finer).
    pub scale: f32,
}

/// Something the player can act on. `rect` is in game coordinates (y up); `walk_to` is where the
/// player stands to do it.
#[derive(Clone, Debug)]
pub struct Hotspot {
    pub id: String,
    pub name: String,
    pub rect: Rect,
    pub walk_to: Option<Vec2>,
}

/// A room: a walkable outline (possibly concave) with obstacle holes, plus the images drawn for it.
/// Walking is done on the outline shrunk, and the obstacles grown, by a margin, so a character
/// standing on the walkable boundary does not overlap walls or obstacles.
pub struct Room {
    outline: Vec<Vec2>,
    obstacles: Vec<Vec<Vec2>>,
    walk_outline: Vec<Vec2>,
    walk_obstacles: Vec<Vec<Vec2>>,
    background: Option<String>,
    props: Vec<Prop>,
    hotspots: Vec<Hotspot>,
    dark: bool,
    vertices: Vec<Vec2>,
    edges: Vec<Segment>,
    corners_visible: Vec<Vec<bool>>,
}

impl Room {
    pub fn new(outline: Vec<Vec2>, obstacles: Vec<Vec<Vec2>>) -> Self {
        Self::with_options(outline, obstacles, 0.0, None, Vec::new())
    }

    pub fn with_options(
        outline: Vec<Vec2>,
        obstacles: Vec<Vec<Vec2>>,
        margin: f32,
        background: Option<String>,
        props: Vec<Prop>,
    ) -> Self {
        let walk_outline = offset(&outline, -margin);
        let walk_obstacles: Vec<Vec<Vec2>> =
            obstacles.iter().map(|ring| offset(ring, margin)).collect();

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        for ring in std::iter::once(&walk_outline).chain(&walk_obstacles) {
            for (i, &a) in ring.iter().enumerate() {
                vertices.push(a);
                edges.push(Segment {
                    a,
                    b: ring[(i + 1) % ring.len()],
                });
            }
        }

        let mut room = Room {
            outline,
            obstacles,
            walk_outline,
            walk_obstacles,
            background,
            props,
            hotspots: Vec::new(),
            dark: false,
            vertices,
            edges,
            corners_visible: Vec::new(),
        };
        room.corners_visible = pathfinder::corner_visibility(&room);
        room
    }

    /// A dark room is drawn dark until a script turns its lights on (`World::set_light`).
    pub fn with_dark(mut self, dark: bool) -> Self {
        self.dark = dark;
        self
    }

    pub fn with_hotspots(mut self, hotspots: Vec<Hotspot>) -> Self {
        self.hotspots = hotspots;
        self
    }

    /// Reads a room from the table a `rooms/*.lua` file returns.
    pub fn from_lua(source: &str, name: &str) -> mlua::Result<Self> {
        let lua = crate::script::data_lua()?;
        let table: Table = lua.load(source).set_name(name).eval()?;

        let outline = read_points(&table.get("walkArea")?)?;
        let obstacles = list(&table, "obstacles", |ring| read_points(&ring))?;
        let props = list(&table, "props", |prop| {
            let y: f32 = prop.get("y")?;
            Ok(Prop {
                id: prop.get("id")?,
                image: prop.get("image")?,
                x: prop.get("x")?,
                y,
                depth: prop.get::<Option<f32>>("depth")?.unwrap_or(y),
                frames: prop.get::<Option<usize>>("frames")?.unwrap_or(1).max(1),
                fps: prop.get::<Option<f32>>("fps")?.unwrap_or(DEFAULT_FPS),
                idle: prop.get::<Option<usize>>("idle")?.unwrap_or(0),
                glow: prop.get::<Option<bool>>("glow")?.unwrap_or(false),
                scale: prop
                    .get::<Option<f32>>("scale")?
                    .filter(|scale| scale.is_finite() && *scale > 0.0)
                    .unwrap_or(1.0),
            })
        })?;
        let hotspots = list(&table, "hotspots", |hotspot| {
            let rect: Table = hotspot.get("rect")?;
            let walk_to: Option<Table> = hotspot.get("walkTo")?;
            Ok(Hotspot {
                id: hotspot.get("id")?,
                name: hotspot.get("name")?,
                rect: Rect::new(rect.get(1)?, rect.get(2)?, rect.get(3)?, rect.get(4)?),
                walk_to: walk_to
                    .map(|point| Ok::<_, mlua::Error>(vec2(point.get(1)?, point.get(2)?)))
                    .transpose()?,
            })
        })?;

        let margin = table.get::<Option<f32>>("margin")?.unwrap_or(0.0);
        let dark = table.get::<Option<bool>>("dark")?.unwrap_or(false);
        Ok(
            Self::with_options(outline, obstacles, margin, table.get("background")?, props)
                .with_hotspots(hotspots)
                .with_dark(dark),
        )
    }

    pub fn is_dark(&self) -> bool {
        self.dark
    }

    pub fn hotspots(&self) -> &[Hotspot] {
        &self.hotspots
    }

    /// The hotspot under a point; later entries in the room file are on top.
    pub fn hotspot_at(&self, point: Vec2) -> Option<&Hotspot> {
        self.hotspots
            .iter()
            .rev()
            .find(|hotspot| hotspot.rect.contains(point))
    }

    pub fn background(&self) -> Option<&str> {
        self.background.as_deref()
    }

    pub fn props(&self) -> &[Prop] {
        &self.props
    }

    pub fn outline(&self) -> &[Vec2] {
        &self.outline
    }

    pub fn obstacles(&self) -> &[Vec<Vec2>] {
        &self.obstacles
    }

    pub fn walk_outline(&self) -> &[Vec2] {
        &self.walk_outline
    }

    pub fn walk_obstacles(&self) -> &[Vec<Vec2>] {
        &self.walk_obstacles
    }

    /// Corners of the walkable boundary, which paths bend around.
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }

    pub fn edges(&self) -> &[Segment] {
        &self.edges
    }

    pub(crate) fn corners_visible(&self, i: usize, j: usize) -> bool {
        self.corners_visible[i][j]
    }

    /// The boundary counts as walkable, so paths can run along walls and around obstacle corners.
    pub fn is_walkable(&self, point: Vec2) -> bool {
        if point.distance(self.nearest_boundary_point(point)) < BOUNDARY_EPS {
            return true;
        }
        if !point_in_polygon(&self.walk_outline, point) {
            return false;
        }
        !self
            .walk_obstacles
            .iter()
            .any(|ring| point_in_polygon(ring, point))
    }

    fn nearest_boundary_point(&self, point: Vec2) -> Vec2 {
        let mut best = Vec2::ZERO;
        let mut best_distance = f32::MAX;
        for edge in &self.edges {
            let nearest = nearest_point_on_segment(edge.a, edge.b, point);
            let distance = nearest.distance_squared(point);
            if distance < best_distance {
                best_distance = distance;
                best = nearest;
            }
        }
        best
    }

    /// Returns the point itself if it is walkable, otherwise the nearest point on the walkable boundary.
    pub fn constrain(&self, x: f32, y: f32) -> Vec2 {
        let point = vec2(x, y);
        if self.is_walkable(point) {
            point
        } else {
            self.nearest_boundary_point(point)
        }
    }

    /// Waypoints for walking from a point to the nearest walkable point to (x, y); empty if unreachable.
    pub fn find_path(&self, from: Vec2, x: f32, y: f32) -> Vec<Vec2> {
        pathfinder::find(self, from, self.constrain(x, y)).unwrap_or_default()
    }
}

/// Reads each table in the optional list `key` of a room file with `read`.
fn list<T>(
    table: &Table,
    key: &str,
    read: impl Fn(Table) -> mlua::Result<T>,
) -> mlua::Result<Vec<T>> {
    let items: Option<Table> = table.get(key)?;
    items.map_or(Ok(Vec::new()), |items| {
        items
            .sequence_values::<Table>()
            .map(|item| read(item?))
            .collect()
    })
}

/// A ring of at least three finite points.
fn read_points(points: &Table) -> mlua::Result<Vec<Vec2>> {
    let ring: Vec<Vec2> = points
        .sequence_values::<Table>()
        .map(|point| {
            let point = point?;
            let x: f32 = point.get(1)?;
            let y: f32 = point.get(2)?;
            Ok(vec2(x, y))
        })
        .collect::<mlua::Result<_>>()?;
    if ring.len() < 3 || ring.iter().any(|p| !p.is_finite()) {
        return Err(mlua::Error::runtime(
            "an outline needs at least 3 finite points",
        ));
    }
    Ok(ring)
}

/// Moves every edge of the ring outwards by the distance (inwards if negative), mitring the corners.
pub fn offset(ring: &[Vec2], distance: f32) -> Vec<Vec2> {
    if distance == 0.0 {
        return ring.to_vec();
    }
    let count = ring.len();
    let double_area: f32 = (0..count)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % count]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let outward = if double_area > 0.0 { 1.0 } else { -1.0 };

    (0..count)
        .map(|i| {
            let point = ring[i];
            let previous = ring[(i + count - 1) % count];
            let next = ring[(i + 1) % count];
            let incoming = (point - previous).normalize_or_zero();
            let outgoing = (next - point).normalize_or_zero();
            let normal_in = vec2(incoming.y, -incoming.x) * outward;
            let normal_out = vec2(outgoing.y, -outgoing.x) * outward;
            let denominator = (1.0 + normal_in.dot(normal_out)).max(MIN_MITER_DENOMINATOR);
            point + (normal_in + normal_out) * (distance / denominator)
        })
        .collect()
}

fn nearest_point_on_segment(a: Vec2, b: Vec2, point: Vec2) -> Vec2 {
    let ab = b - a;
    let length2 = ab.length_squared();
    if length2 == 0.0 {
        return a;
    }
    a + ab * ((point - a).dot(ab) / length2).clamp(0.0, 1.0)
}

fn point_in_polygon(ring: &[Vec2], point: Vec2) -> bool {
    let mut inside = false;
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % ring.len()];
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
    }
    inside
}
