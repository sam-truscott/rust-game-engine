//! The panel along the bottom of the screen: the inventory slots.
//!
//! Everything here is in screen coordinates (320x180, y up). The room is drawn above the panel, shifted
//! up by `PANEL_HEIGHT`, so a point in the room is a screen point minus `PANEL_HEIGHT` in y.

use macroquad::math::{Rect, Vec2, vec2};

pub const SCREEN_WIDTH: f32 = 320.0;
pub const SCREEN_HEIGHT: f32 = 180.0;

/// The panel fills the screen below this y; clicks there never walk the player.
pub const PANEL_HEIGHT: f32 = 26.0;
pub const SLOT_COUNT: usize = 11;

const SLOT_WIDTH: f32 = 24.0;
const SLOT_HEIGHT: f32 = 20.0;
const SLOT_GAP: f32 = 4.0;
const SLOTS_BOTTOM: f32 = 3.0;

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
