//! The verb bar: a strip across the top of the screen, over the wall, in game coordinates (y up).

use macroquad::math::{Rect, Vec2};

use crate::room::Hotspot;

pub const SCREEN_WIDTH: f32 = 320.0;
pub const SCREEN_HEIGHT: f32 = 180.0;

/// Everything at or above this y belongs to the verb bar; clicks there never walk the player.
pub const PANEL_BOTTOM: f32 = 146.0;
pub const SENTENCE_BASELINE: f32 = 168.0;

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

/// What the bar says: the chosen verb, and the name of what the cursor is over.
pub fn sentence(verb: Verb, hovered: Option<&Hotspot>) -> String {
    match hovered {
        Some(hotspot) => format!("{} {}", verb.label(), hotspot.name),
        None => verb.label().to_owned(),
    }
}
