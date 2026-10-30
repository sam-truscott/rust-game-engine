use game::actor::Actor;
use game::assets;
use game::direction::Direction;
use game::room::{Prop, Room};
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
}

impl RoomRenderer {
    pub fn new(room: &Room) -> Self {
        RoomRenderer {
            background: room.background().map(load_texture),
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
    let frame = prop.id.as_deref().map_or(0, |id| world.frame(id));
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
