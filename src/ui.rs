//! The panel along the bottom of the screen: the verb buttons above the inventory slots.
//!
//! Everything here is in screen coordinates (320x180, y up). The room is drawn above the panel, shifted
//! up by `PANEL_HEIGHT`, so a point in the room is a screen point minus `PANEL_HEIGHT` in y.

use macroquad::math::{Rect, Vec2, vec2};

pub const SCREEN_WIDTH: f32 = 320.0;
pub const SCREEN_HEIGHT: f32 = 180.0;

/// The panel fills the screen below this y; clicks there never walk the player.
pub const PANEL_HEIGHT: f32 = 40.0;
pub const SLOT_COUNT: usize = 11;

const SLOT_WIDTH: f32 = 24.0;
const SLOT_HEIGHT: f32 = 20.0;
const SLOT_GAP: f32 = 4.0;
const SLOTS_BOTTOM: f32 = 3.0;

const BUTTON_WIDTH: f32 = 48.0;
const BUTTON_HEIGHT: f32 = 12.0;
const BUTTON_GAP: f32 = 4.0;
const BUTTONS_BOTTOM: f32 = 26.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    WalkTo,
    LookAt,
    PickUp,
    Use,
    TalkTo,
}

impl Verb {
    pub const ALL: [Verb; 5] = [
        Verb::WalkTo,
        Verb::LookAt,
        Verb::PickUp,
        Verb::Use,
        Verb::TalkTo,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Verb::WalkTo => "Walk to",
            Verb::LookAt => "Look at",
            Verb::PickUp => "Pick up",
            Verb::Use => "Use",
            Verb::TalkTo => "Talk to",
        }
    }

    /// The name scripts use: `hotspot.counter.look`, `hotspot.counter.pickup`, ...
    pub fn key(self) -> &'static str {
        match self {
            Verb::WalkTo => "walk",
            Verb::LookAt => "look",
            Verb::PickUp => "pickup",
            Verb::Use => "use",
            Verb::TalkTo => "talk",
        }
    }
}

pub fn button_rect(verb: Verb) -> Rect {
    let index = Verb::ALL.iter().position(|&v| v == verb).unwrap_or(0) as f32;
    Rect::new(
        BUTTON_GAP + index * (BUTTON_WIDTH + BUTTON_GAP),
        BUTTONS_BOTTOM,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
    )
}

pub fn verb_at(point: Vec2) -> Option<Verb> {
    Verb::ALL
        .into_iter()
        .find(|&verb| button_rect(verb).contains(point))
}

pub fn in_panel(point: Vec2) -> bool {
    point.y < PANEL_HEIGHT
}

/// A screen point as a point in the room.
pub fn to_room(screen: Vec2) -> Vec2 {
    vec2(screen.x, screen.y - PANEL_HEIGHT)
}

/// A point in the room as a screen point.
pub fn to_screen(room: Vec2) -> Vec2 {
    vec2(room.x, room.y + PANEL_HEIGHT)
}

pub fn slot_rect(index: usize) -> Rect {
    Rect::new(
        SLOT_GAP + index as f32 * (SLOT_WIDTH + SLOT_GAP),
        SLOTS_BOTTOM,
        SLOT_WIDTH,
        SLOT_HEIGHT,
    )
}

/// The occupied slot under a point, given how many items are carried.
pub fn slot_at(point: Vec2, count: usize) -> Option<usize> {
    (0..count.min(SLOT_COUNT)).find(|&index| slot_rect(index).contains(point))
}
