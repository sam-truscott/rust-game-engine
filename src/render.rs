use game::actor::Actor;
use game::assets;
use game::direction::Direction;
use game::room::{Prop, Room, StreetView};
use game::street::Car;
use game::ui::{RENDER_SCALE, SCREEN_HEIGHT, SCREEN_WIDTH};
use game::world::World;
use macroquad::prelude::*;

pub const FRAME_WIDTH: f32 = 24.0;
pub const FRAME_HEIGHT: f32 = 48.0;

const WALK_FRAMES: usize = 6;
/// Sheet columns: idle, then the walk cycle; rows: front, back, side.
const SHEET_COLUMNS: f32 = (WALK_FRAMES + 1) as f32;
const SHEET_ROWS: f32 = 3.0;
const DISTANCE_PER_FRAME: f32 = 7.5;
/// What covers an unlit room; almost opaque.
const DARKNESS: Color = Color::new(0.0, 0.0, 0.0, 0.99);

fn load_texture(path: &str) -> Texture2D {
    let bytes = assets::bytes(path).unwrap_or_else(|| panic!("missing asset {path}"));
    let texture = Texture2D::from_file_with_format(&bytes, None);
    texture.set_filter(FilterMode::Nearest);
    texture
}

/// Everything here draws into the virtual screen: origin top-left, y down. Game coordinates are
/// y up, so positions are flipped with `SCREEN_HEIGHT` on the way in.
///
/// Draws an actor from a sheet with one row per view (front, back, right-facing side) and one column
/// per frame (idle, then the walk cycle). The left view is the side view mirrored. The walk frame
/// follows the distance walked, so the feet do not slide.
pub struct CharacterSprite {
    texture: Texture2D,
}

impl CharacterSprite {
    pub fn new(path: &str) -> Self {
        CharacterSprite {
            texture: load_texture(path),
        }
    }

    /// Draws the actor standing with its feet at its position, centred horizontally.
    pub fn draw(&self, actor: &Actor) {
        let facing = actor.facing();
        let row = match facing {
            Direction::Front => 0,
            Direction::Back => 1,
            Direction::Left | Direction::Right => 2,
        };
        let column = if actor.is_walking() {
            1 + (actor.walk_distance() / DISTANCE_PER_FRAME) as usize % WALK_FRAMES
        } else {
            0
        };

        // The sheet may be drawn at any multiple of the game's resolution; frames are cut by count.
        let frame = vec2(
            self.texture.width() / SHEET_COLUMNS,
            self.texture.height() / SHEET_ROWS,
        );
        let position = actor.position();
        blit(
            &self.texture,
            vec2(
                position.x - FRAME_WIDTH / 2.0,
                SCREEN_HEIGHT - position.y - FRAME_HEIGHT,
            ),
            Rect::new(
                column as f32 * frame.x,
                row as f32 * frame.y,
                frame.x,
                frame.y,
            ),
            vec2(FRAME_WIDTH, FRAME_HEIGHT),
            facing == Direction::Left,
        );
    }
}

enum Item<'a> {
    Prop(&'a Prop, &'a Texture2D),
    Actor,
}

/// Draws a room's background and props, interleaved with the actor by depth.
pub struct RoomRenderer {
    background: Option<Texture2D>,
    props: Vec<(Prop, Texture2D)>,
    street: Option<StreetView>,
}

impl RoomRenderer {
    pub fn new(room: &Room) -> Self {
        RoomRenderer {
            background: room.background().map(load_texture),
            street: room.street().copied(),
            props: room
                .props()
                .iter()
                .map(|prop| (prop.clone(), load_texture(&prop.image)))
                .collect(),
        }
    }

    /// Items with a larger depth are further back and drawn first; ties draw the actor last.
    /// Props a script has hidden are skipped. In a dark room everything is covered by darkness except
    /// the glowing props, which are drawn over it.
    pub fn draw(&self, actor: &Actor, sprite: &CharacterSprite, world: &World, dark: bool) {
        if let Some(background) = &self.background {
            draw_texture_ex(
                background,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(SCREEN_WIDTH, SCREEN_HEIGHT)),
                    ..Default::default()
                },
            );
        }

        let car = self.street.as_ref().zip(world.car());
        if let Some((view, car)) = &car {
            draw_car(car, view, self.background.as_ref());
            draw_light(car, view);
        }

        let shown = |prop: &Prop| !prop.id.as_deref().is_some_and(|id| world.is_hidden(id));
        let mut items: Vec<(f32, Item)> = self
            .props
            .iter()
            .filter(|(prop, _)| shown(prop) && !(dark && prop.glow))
            .map(|(prop, texture)| (prop.depth, Item::Prop(prop, texture)))
            .collect();
        items.push((actor.position().y, Item::Actor));
        items.sort_by(|a, b| b.0.total_cmp(&a.0));

        for (_, item) in items {
            match item {
                Item::Prop(prop, texture) => draw_prop(prop, texture, world),
                Item::Actor => sprite.draw(actor),
            }
        }

        if let Some((_, car)) = &car {
            draw_wash(car);
        }

        if dark {
            draw_rectangle(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DARKNESS);
            for (prop, texture) in self
                .props
                .iter()
                .filter(|(prop, _)| prop.glow && shown(prop))
            {
                draw_prop(prop, texture, world);
            }
        }
    }
}

/// Draws `source`, a pixel rectangle of the texture, `size` game units large with its top-left at `at`
/// (rounded to whole pixels of the render target).
fn blit(texture: &Texture2D, at: Vec2, source: Rect, size: Vec2, flip_x: bool) {
    let snap = |value: f32| (value * RENDER_SCALE as f32).round() / RENDER_SCALE as f32;
    draw_texture_ex(
        texture,
        snap(at.x),
        snap(at.y),
        WHITE,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(size),
            flip_x,
            ..Default::default()
        },
    );
}

fn draw_prop(prop: &Prop, texture: &Texture2D, world: &World) {
    let width = texture.width() / prop.frames as f32;
    let frame = prop.id.as_deref().map_or(0, |id| {
        world.idle_frame(id, prop.idle, prop.fps, prop.x * 0.37)
    });
    blit(
        texture,
        vec2(
            prop.x,
            SCREEN_HEIGHT - prop.y - texture.height() / prop.scale,
        ),
        Rect::new(
            frame.min(prop.frames - 1) as f32 * width,
            0.0,
            width,
            texture.height(),
        ),
        vec2(width, texture.height()) / prop.scale,
        false,
    );
}

/// The mouse pointer, drawn at window resolution by `main.rs` in place of the system cursor. The
/// image's top-left pixel is the point that clicks.
pub struct CursorSprite {
    texture: Texture2D,
}

impl CursorSprite {
    pub fn new(path: &str) -> Self {
        CursorSprite {
            texture: load_texture(path),
        }
    }

    /// The image size in game units (the image has `RENDER_SCALE` pixels per unit).
    pub fn size(&self) -> Vec2 {
        vec2(self.texture.width(), self.texture.height()) / RENDER_SCALE as f32
    }

    /// Draws the pointer with its hot spot at a window point, each pixel of the image `scale` window
    /// pixels wide.
    pub fn draw(&self, window: Vec2, scale: f32, translucent: bool) {
        let tint = Color::new(1.0, 1.0, 1.0, if translucent { 0.4 } else { 1.0 });
        draw_texture_ex(
            &self.texture,
            window.x.round(),
            window.y.round(),
            tint,
            DrawTextureParams {
                dest_size: Some(self.size() * scale),
                ..Default::default()
            },
        );
    }
}

/// Art pixel of the car, in game units (the room art is drawn in 5x5 blocks of the 1280x720 picture).
const BLOCK: f32 = 1.25;
const CAR_PATTERN: [&str; 4] = ["..BBBBB...", ".BCCBCCB..", "TBBBBBBBBH", ".WW....WW."];
const CAR_BODIES: [Color; 3] = [
    Color::new(0.10, 0.13, 0.20, 1.0),
    Color::new(0.30, 0.09, 0.10, 1.0),
    Color::new(0.12, 0.20, 0.17, 1.0),
];
const HEADLIGHT: Color = Color::new(1.0, 0.86, 0.55, 1.0);
const LIGHT_BANDS: usize = 14;

/// Draws a filled rectangle given in room coordinates (y up), cut to `clip` when there is one.
fn room_rect(x: f32, y: f32, w: f32, h: f32, clip: Option<Rect>, color: Color) {
    let (mut x0, mut x1, mut y0, mut y1) = (x, x + w, y, y + h);
    if let Some(clip) = clip {
        x0 = x0.max(clip.x);
        x1 = x1.min(clip.x + clip.w);
        y0 = y0.max(clip.y);
        y1 = y1.min(clip.y + clip.h);
    }
    if x1 > x0 && y1 > y0 {
        draw_rectangle(x0, SCREEN_HEIGHT - y1, x1 - x0, y1 - y0, color);
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

/// How bright the room is lit by a car's headlights: nothing as it enters the window, most as it passes.
fn glow(car: &Car) -> f32 {
    (car.progress * std::f32::consts::PI).sin()
}

/// The x of a car's front when it is `progress` of the way across `glass`.
fn car_x(car: &Car, glass: Rect) -> f32 {
    let width = CAR_PATTERN[0].len() as f32 * BLOCK;
    let along = if car.rightwards {
        car.progress
    } else {
        1.0 - car.progress
    };
    glass.x - width + along * (glass.w + 2.0 * width)
}

/// The car on the street, seen through the lower pane of the window.
fn draw_car(car: &Car, view: &StreetView, background: Option<&Texture2D>) {
    let glass = view.glass;
    let left = car_x(car, glass);
    let body = CAR_BODIES[car.style % CAR_BODIES.len()];
    let width = CAR_PATTERN[0].len();
    for (row, line) in CAR_PATTERN.iter().enumerate() {
        for (column, cell) in line.chars().enumerate() {
            let color = match cell {
                'B' => body,
                'C' => Color::new(0.20, 0.25, 0.38, 1.0),
                'H' => HEADLIGHT,
                'T' => Color::new(0.85, 0.12, 0.10, 1.0),
                'W' => Color::new(0.02, 0.02, 0.04, 1.0),
                _ => continue,
            };
            // The pattern faces right; mirror it for a car driving left.
            let column = if car.rightwards {
                column
            } else {
                width - 1 - column
            };
            room_rect(
                left + column as f32 * BLOCK,
                glass.y + (CAR_PATTERN.len() - 1 - row) as f32 * BLOCK,
                BLOCK,
                BLOCK,
                Some(glass),
                color,
            );
        }
    }
    // A soft glow around the headlight on the glass.
    let front = if car.rightwards {
        left + width as f32 * BLOCK
    } else {
        left
    };
    let ahead = if car.rightwards { 1.0 } else { -1.0 };
    for (step, alpha) in [(1.0, 0.22), (2.0, 0.12), (3.0, 0.06)] {
        room_rect(
            front.min(front + ahead * step * BLOCK * 2.0),
            glass.y,
            step * BLOCK * 2.0,
            BLOCK * 3.0,
            Some(glass),
            with_alpha(HEADLIGHT, alpha),
        );
    }
    // The bar between the two panes is in front of the car: put it back from the picture.
    if let Some(background) = background {
        let scale = background.width() / SCREEN_WIDTH;
        let (x, w) = (glass.x + glass.w * 0.4, BLOCK);
        let bar = Rect::new(
            x * scale,
            (SCREEN_HEIGHT - glass.y - glass.h) * scale,
            w * scale,
            glass.h * scale,
        );
        draw_texture_ex(
            background,
            x,
            SCREEN_HEIGHT - glass.y - glass.h,
            WHITE,
            DrawTextureParams {
                source: Some(bar),
                dest_size: Some(vec2(w, glass.h)),
                ..Default::default()
            },
        );
    }
}

/// The light that comes in through the window: a beam falling down the wall and over the floor,
/// sliding along as the car passes, and a glint in the mirror.
fn draw_light(car: &Car, view: &StreetView) {
    let strength = glow(car);
    let glass = view.glass;
    let ahead = if car.rightwards { 1.0 } else { -1.0 };
    let centre = glass.x
        + glass.w
            * if car.rightwards {
                car.progress
            } else {
                1.0 - car.progress
            };
    let top = glass.y;
    let depth = top - view.floor;
    for band in 0..LIGHT_BANDS {
        let t = band as f32 / LIGHT_BANDS as f32;
        let y = top - (band + 1) as f32 * depth / LIGHT_BANDS as f32;
        // Quantised, so the light falls off in steps like the dithered pools in the art.
        let alpha = ((1.0 - t).powf(1.4) * strength * 0.30 * 10.0).round() / 10.0;
        let middle = centre + ahead * depth * 0.45 * t;
        let half = glass.w * 0.18 + depth * 0.22 * t;
        let snap = |value: f32| (value / BLOCK).round() * BLOCK;
        room_rect(
            snap(middle - half),
            snap(y),
            snap(half * 2.0),
            snap(depth / LIGHT_BANDS as f32).max(BLOCK),
            None,
            with_alpha(Color::new(1.0, 0.7, 0.35, 1.0), alpha),
        );
    }
    if let Some(mirror) = view.mirror {
        let stripe = mirror.w * 0.22;
        let across = (mirror.w + stripe)
            * if car.rightwards {
                car.progress
            } else {
                1.0 - car.progress
            };
        let x = mirror.x + across - stripe;
        room_rect(
            x,
            mirror.y,
            stripe,
            mirror.h,
            Some(mirror),
            with_alpha(Color::new(1.0, 0.9, 0.7, 1.0), strength * 0.35),
        );
    }
}

/// A faint warm wash over everything, so the whole room brightens a little.
fn draw_wash(car: &Car) {
    draw_rectangle(
        0.0,
        0.0,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        Color::new(1.0, 0.8, 0.5, glow(car) * 0.05),
    );
}
