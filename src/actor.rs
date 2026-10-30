use std::collections::VecDeque;

use macroquad::math::Vec2;

use crate::direction::Direction;
use crate::room::Room;

const WALK_SPEED: f32 = 60.0;
const SPEECH_BASE_SECONDS: f32 = 1.0;
const SPEECH_SECONDS_PER_CHAR: f32 = 0.05;
const MIN_MOVE: f32 = 1e-3;

pub struct Actor {
    position: Vec2,
    path: VecDeque<Vec2>,
    facing: Direction,
    walk_distance: f32,
    speech: Option<String>,
    speech_remaining: f32,
}

impl Actor {
    pub fn new(x: f32, y: f32) -> Self {
        Actor {
            position: Vec2::new(x, y),
            path: VecDeque::new(),
            facing: Direction::Front,
            walk_distance: 0.0,
            speech: None,
            speech_remaining: 0.0,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn facing(&self) -> Direction {
        self.facing
    }

    /// Distance walked since the current walk began, so animation can follow the feet.
    pub fn walk_distance(&self) -> f32 {
        self.walk_distance
    }

    pub fn speech(&self) -> Option<&str> {
        self.speech.as_deref()
    }

    pub fn walk_to(&mut self, room: &Room, x: f32, y: f32) {
        if self.path.is_empty() {
            self.walk_distance = 0.0;
        }
        self.path = room.find_path(self.position, x, y).into();
    }

    pub fn is_walking(&self) -> bool {
        !self.path.is_empty()
    }

    pub fn say(&mut self, text: &str) {
        self.speech = Some(text.to_owned());
        self.speech_remaining =
            SPEECH_BASE_SECONDS + SPEECH_SECONDS_PER_CHAR * text.chars().count() as f32;
    }

    pub fn set_facing(&mut self, facing: Direction) {
        self.facing = facing;
    }

    pub fn stop_speaking(&mut self) {
        self.speech = None;
    }

    pub fn is_speaking(&self) -> bool {
        self.speech.is_some()
    }

    fn face(&mut self, dx: f32, dy: f32) {
        self.facing = if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if dy > 0.0 {
            Direction::Back
        } else {
            Direction::Front
        };
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let mut step = WALK_SPEED * delta_seconds;
        while step > 0.0 {
            let Some(&next) = self.path.front() else {
                break;
            };
            let distance = self.position.distance(next);
            if distance > MIN_MOVE {
                self.face(next.x - self.position.x, next.y - self.position.y);
            }
            if distance <= step {
                self.position = next;
                self.path.pop_front();
                self.walk_distance += distance;
                step -= distance;
            } else {
                self.position += (next - self.position).normalize() * step;
                self.walk_distance += step;
                step = 0.0;
            }
        }
        if self.speech.is_some() {
            self.speech_remaining -= delta_seconds;
            if self.speech_remaining <= 0.0 {
                self.speech = None;
            }
        }
    }
}
