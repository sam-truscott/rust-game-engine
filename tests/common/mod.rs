//! Helpers shared by the integration tests.
#![allow(dead_code)]

use game::room::Room;
use macroquad::math::{Vec2, vec2};

pub fn ring(points: &[(f32, f32)]) -> Vec<Vec2> {
    points.iter().map(|&(x, y)| vec2(x, y)).collect()
}

/// A 200x100 rectangle.
pub fn floor_outline() -> Vec<Vec2> {
    ring(&[(0.0, 0.0), (200.0, 0.0), (200.0, 100.0), (0.0, 100.0)])
}

/// An empty room with that outline.
pub fn floor() -> Room {
    Room::new(floor_outline(), vec![])
}

pub fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}
