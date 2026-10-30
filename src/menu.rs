//! The save/load menu (F2): a button to save, a button to close and a list of saves to load.
//! Layout is in screen coordinates (320x180, y up) like `ui`.

use macroquad::math::{Rect, Vec2};

use crate::save::SaveEntry;

pub const PANEL: Rect = Rect {
    x: 40.0,
    y: 20.0,
    w: 240.0,
    h: 140.0,
};
pub const VISIBLE_ROWS: usize = 7;
const ROW_HEIGHT: f32 = 12.0;
const ROW_PITCH: f32 = 14.0;
const FIRST_ROW_BOTTOM: f32 = 116.0;

pub fn save_button() -> Rect {
    Rect::new(48.0, 132.0, 108.0, ROW_HEIGHT)
}

pub fn close_button() -> Rect {
    Rect::new(164.0, 132.0, 108.0, ROW_HEIGHT)
}

/// The rectangle of the n-th visible row of the list, counting from the top.
pub fn row_rect(row: usize) -> Rect {
    Rect::new(
        48.0,
        FIRST_ROW_BOTTOM - row as f32 * ROW_PITCH,
        224.0,
        ROW_HEIGHT,
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum MenuAction {
    /// The click hit nothing.
    None,
    Close,
    /// Make a new numbered save of the current game.
    Save,
    /// Replace the current game with this save (a `SaveEntry::name`).
    Load(String),
}

#[derive(Default)]
pub struct SaveMenu {
    entries: Vec<SaveEntry>,
    first: usize,
    /// A line about the last action ("Saved as Save 3"), shown under the buttons.
    pub message: Option<String>,
}

impl SaveMenu {
    pub fn new(entries: Vec<SaveEntry>) -> Self {
        SaveMenu {
            entries,
            first: 0,
            message: None,
        }
    }

    pub fn set_entries(&mut self, entries: Vec<SaveEntry>) {
        self.entries = entries;
        self.first = self.first.min(self.max_first());
    }

    pub fn entries(&self) -> &[SaveEntry] {
        &self.entries
    }

    /// The entries in view, from the top row down.
    pub fn visible(&self) -> &[SaveEntry] {
        let end = (self.first + VISIBLE_ROWS).min(self.entries.len());
        &self.entries[self.first..end]
    }

    fn max_first(&self) -> usize {
        self.entries.len().saturating_sub(VISIBLE_ROWS)
    }

    /// Scrolls the list by rows (positive is down), stopping at the ends.
    pub fn scroll(&mut self, rows: isize) {
        self.first = self.first.saturating_add_signed(rows).min(self.max_first());
    }

    pub fn click(&self, point: Vec2) -> MenuAction {
        if save_button().contains(point) {
            return MenuAction::Save;
        }
        if close_button().contains(point) {
            return MenuAction::Close;
        }
        self.visible()
            .iter()
            .enumerate()
            .find(|(row, _)| row_rect(*row).contains(point))
            .map_or(MenuAction::None, |(_, entry)| {
                MenuAction::Load(entry.name.clone())
            })
    }
}
