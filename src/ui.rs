//! The verb bar across the top of the screen (over the wall) and the inventory strip along the
//! bottom, in game coordinates (y up).

use macroquad::math::{Rect, Vec2};

pub const SCREEN_WIDTH: f32 = 320.0;
pub const SCREEN_HEIGHT: f32 = 180.0;

/// Everything at or above this y belongs to the verb bar; clicks there never walk the player.
pub const PANEL_BOTTOM: f32 = 146.0;
pub const SENTENCE_BASELINE: f32 = 168.0;

/// Everything below this y belongs to the inventory strip.
pub const INVENTORY_TOP: f32 = 24.0;
pub const SLOT_COUNT: usize = 11;

const SLOT_WIDTH: f32 = 24.0;
const SLOT_HEIGHT: f32 = 20.0;
const SLOT_GAP: f32 = 4.0;
const SLOTS_BOTTOM: f32 = 2.0;

const BUTTON_WIDTH: f32 = 48.0;
const BUTTON_HEIGHT: f32 = 12.0;
const BUTTON_GAP: f32 = 4.0;
const BUTTONS_BOTTOM: f32 = 150.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    WalkTo,
    LookAt,
    PickUp,
    Use,
    Open,
    TalkTo,
}

impl Verb {
    pub const ALL: [Verb; 6] = [
        Verb::WalkTo,
        Verb::LookAt,
        Verb::PickUp,
        Verb::Use,
        Verb::Open,
        Verb::TalkTo,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Verb::WalkTo => "Walk to",
            Verb::LookAt => "Look at",
            Verb::PickUp => "Pick up",
            Verb::Use => "Use",
            Verb::Open => "Open",
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
            Verb::Open => "open",
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
    point.y >= PANEL_BOTTOM
}

pub fn in_inventory(point: Vec2) -> bool {
    point.y < INVENTORY_TOP
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

/// What the bar says: the chosen verb (or the item being used), and what the cursor is over.
pub fn sentence(verb: Verb, held: Option<&str>, target: Option<&str>) -> String {
    let start = match held {
        Some(item) => format!("Use {item} with"),
        None => verb.label().to_owned(),
    };
    match target {
        Some(name) => format!("{start} {name}"),
        None => start,
    }
}
