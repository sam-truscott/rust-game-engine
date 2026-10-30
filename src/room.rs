use macroquad::math::{Vec2, vec2};
use mlua::{Lua, Table};

use crate::pathfinder;

const BOUNDARY_EPS: f32 = 0.01;
const MIN_MITER_DENOMINATOR: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
}

/// An image placed in the room; characters with a smaller y than the depth are drawn in front of it.
#[derive(Clone, Debug)]
pub struct Prop {
    pub image: String,
    pub x: f32,
    pub y: f32,
    pub depth: f32,
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
            vertices,
            edges,
            corners_visible: Vec::new(),
        };
        room.corners_visible = pathfinder::corner_visibility(&room);
        room
    }

    /// Reads a room from the table a `rooms/*.lua` file returns.
    pub fn from_lua(source: &str, name: &str) -> mlua::Result<Self> {
        let lua = Lua::new();
        let table: Table = lua.load(source).set_name(name).eval()?;

        let walk_area: Table = table.get("walkArea")?;
        let outline = read_points(&walk_area)?;

        let mut obstacles = Vec::new();
        let obstacle_tables: Option<Table> = table.get("obstacles")?;
        if let Some(obstacle_tables) = obstacle_tables {
            for ring in obstacle_tables.sequence_values::<Table>() {
                obstacles.push(read_points(&ring?)?);
            }
        }

        let mut props = Vec::new();
        let prop_tables: Option<Table> = table.get("props")?;
        if let Some(prop_tables) = prop_tables {
            for prop in prop_tables.sequence_values::<Table>() {
                let prop = prop?;
                let image: String = prop.get("image")?;
                let x: f32 = prop.get("x")?;
                let y: f32 = prop.get("y")?;
                let depth: Option<f32> = prop.get("depth")?;
                props.push(Prop {
                    image,
                    x,
                    y,
                    depth: depth.unwrap_or(y),
                });
            }
        }

        let margin: Option<f32> = table.get("margin")?;
        let background: Option<String> = table.get("background")?;
        Ok(Self::with_options(
            outline,
            obstacles,
            margin.unwrap_or(0.0),
            background,
            props,
        ))
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

fn read_points(points: &Table) -> mlua::Result<Vec<Vec2>> {
    points
        .sequence_values::<Table>()
        .map(|point| {
            let point = point?;
            let x: f32 = point.get(1)?;
            let y: f32 = point.get(2)?;
            Ok(vec2(x, y))
        })
        .collect()
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
