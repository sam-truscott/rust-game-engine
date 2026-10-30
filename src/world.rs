use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Default)]
struct Data {
    inventory: Vec<String>,
    hidden: HashSet<String>,
    frames: HashMap<String, usize>,
    animations: HashMap<String, Animation>,
    lit: HashSet<String>,
}

/// Frames `first` to `last` (either direction) shown one after another, each for 1/fps seconds.
struct Animation {
    first: usize,
    last: usize,
    fps: f32,
    elapsed: f32,
}

impl Animation {
    fn count(&self) -> usize {
        self.first.abs_diff(self.last) + 1
    }

    fn finished(&self) -> bool {
        self.elapsed * self.fps >= self.count() as f32
    }

    fn frame(&self) -> usize {
        let step = ((self.elapsed * self.fps) as usize).min(self.count() - 1);
        if self.last >= self.first {
            self.first + step
        } else {
            self.first - step
        }
    }
}

/// What scripts change: what the player carries, which things in the room are gone and which frame
/// animated props show and which rooms have their lights on.
/// Clones share the same state, so the script runner and the game both see it.
#[derive(Clone, Default)]
pub struct World(Rc<RefCell<Data>>);

impl World {
    /// Adds an item to the inventory; carrying it already does nothing.
    pub fn give(&self, item: &str) {
        let mut data = self.0.borrow_mut();
        if !data.inventory.iter().any(|held| held == item) {
            data.inventory.push(item.to_owned());
        }
    }

    /// Removes an item from the inventory; false if it was not carried.
    pub fn take(&self, item: &str) -> bool {
        let mut data = self.0.borrow_mut();
        let before = data.inventory.len();
        data.inventory.retain(|held| held != item);
        data.inventory.len() != before
    }

    pub fn has(&self, item: &str) -> bool {
        self.0.borrow().inventory.iter().any(|held| held == item)
    }

    /// The carried items, in the order they were picked up.
    pub fn items(&self) -> Vec<String> {
        self.0.borrow().inventory.clone()
    }

    /// Hides the prop and the hotspot with this id.
    pub fn hide(&self, id: &str) {
        self.0.borrow_mut().hidden.insert(id.to_owned());
    }

    /// Turns the lights of a room (by id, e.g. "closet") on or off. Rooms marked `dark` in their file
    /// are drawn dark until their lights are on.
    pub fn set_light(&self, room: &str, on: bool) {
        let mut data = self.0.borrow_mut();
        if on {
            data.lit.insert(room.to_owned());
        } else {
            data.lit.remove(room);
        }
    }

    pub fn is_lit(&self, room: &str) -> bool {
        self.0.borrow().lit.contains(room)
    }

    pub fn is_hidden(&self, id: &str) -> bool {
        self.0.borrow().hidden.contains(id)
    }

    /// The frame of an animated prop to draw: the one playing, else the one it was left on (0 at first).
    pub fn frame(&self, id: &str) -> usize {
        let data = self.0.borrow();
        match data.animations.get(id) {
            Some(animation) => animation.frame(),
            None => data.frames.get(id).copied().unwrap_or(0),
        }
    }

    /// Shows a frame of an animated prop, stopping any animation it was playing.
    pub fn set_frame(&self, id: &str, frame: usize) {
        let mut data = self.0.borrow_mut();
        data.animations.remove(id);
        data.frames.insert(id.to_owned(), frame);
    }

    /// Plays frames `first` to `last` at `fps` frames a second, then stays on `last`.
    pub fn animate(&self, id: &str, first: usize, last: usize, fps: f32) {
        let animation = Animation {
            first,
            last,
            fps: fps.max(0.001),
            elapsed: 0.0,
        };
        let mut data = self.0.borrow_mut();
        data.frames.insert(id.to_owned(), last);
        data.animations.insert(id.to_owned(), animation);
    }

    pub fn is_animating(&self, id: &str) -> bool {
        self.0.borrow().animations.contains_key(id)
    }

    /// Moves animations on; one that has shown its last frame ends.
    pub fn advance(&self, delta_seconds: f32) {
        let mut data = self.0.borrow_mut();
        for animation in data.animations.values_mut() {
            animation.elapsed += delta_seconds;
        }
        data.animations.retain(|_, animation| !animation.finished());
    }
}
