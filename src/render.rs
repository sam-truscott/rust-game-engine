use game::actor::Actor;
use game::assets;
use game::direction::Direction;
use game::room::{Prop, Room};
use game::world::World;
use macroquad::prelude::*;

pub const VIRTUAL_WIDTH: f32 = 320.0;
pub const VIRTUAL_HEIGHT: f32 = 180.0;
pub const FRAME_WIDTH: f32 = 16.0;
pub const FRAME_HEIGHT: f32 = 32.0;

const WALK_FRAMES: usize = 6;
const DISTANCE_PER_FRAME: f32 = 5.0;

fn load_texture(path: &str) -> Texture2D {
    let bytes = assets::bytes(path).unwrap_or_else(|| panic!("missing asset {path}"));
    let texture = Texture2D::from_file_with_format(&bytes, None);
    texture.set_filter(FilterMode::Nearest);
    texture
}

/// Everything here draws into the virtual screen: origin top-left, y down. Game coordinates are
/// y up, so positions are flipped with `VIRTUAL_HEIGHT` on the way in.
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

        let position = actor.position();
        draw_texture_ex(
            &self.texture,
            (position.x - FRAME_WIDTH / 2.0).round(),
            (VIRTUAL_HEIGHT - position.y - FRAME_HEIGHT).round(),
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    column as f32 * FRAME_WIDTH,
                    row as f32 * FRAME_HEIGHT,
                    FRAME_WIDTH,
                    FRAME_HEIGHT,
                )),
                dest_size: Some(vec2(FRAME_WIDTH, FRAME_HEIGHT)),
                flip_x: facing == Direction::Left,
                ..Default::default()
            },
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
    /// Props a script has hidden are skipped.
    pub fn draw(&self, actor: &Actor, sprite: &CharacterSprite, world: &World) {
        if let Some(background) = &self.background {
            draw_texture(background, 0.0, 0.0, WHITE);
        }

        let mut items: Vec<(f32, Item)> = self
            .props
            .iter()
            .filter(|(prop, _)| !prop.id.as_deref().is_some_and(|id| world.is_hidden(id)))
            .map(|(prop, texture)| (prop.depth, Item::Prop(prop, texture)))
            .collect();
        items.push((actor.position().y, Item::Actor));
        items.sort_by(|a, b| b.0.total_cmp(&a.0));

        for (_, item) in items {
            match item {
                Item::Prop(prop, texture) => {
                    let width = texture.width() / prop.frames as f32;
                    let frame = prop.id.as_deref().map_or(0, |id| world.frame(id));
                    draw_texture_ex(
                        texture,
                        prop.x.round(),
                        (VIRTUAL_HEIGHT - prop.y - texture.height()).round(),
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect::new(
                                frame.min(prop.frames - 1) as f32 * width,
                                0.0,
                                width,
                                texture.height(),
                            )),
                            dest_size: Some(vec2(width, texture.height())),
                            ..Default::default()
                        },
                    );
                }
                Item::Actor => sprite.draw(actor),
            }
        }
    }
}
