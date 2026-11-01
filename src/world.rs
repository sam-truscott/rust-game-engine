use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::street::{Car, Street};

#[derive(Default)]
struct Data {
    inventory: Vec<String>,
    hidden: HashSet<String>,
    frames: HashMap<String, usize>,
    animations: HashMap<String, Animation>,
    lit: HashSet<String>,
    flags: HashSet<String>,
    clock: f64,
    street: Street,
    effect: Option<Effect>,
}

/// A whole-screen shader effect in progress.
#[derive(Clone, Debug, PartialEq)]
struct Effect {
    name: String,
    seconds: f32,
    elapsed: f32,
}

/// A screen effect seen from the drawing: which one, how long it lasts and how far it has got.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenEffect {
    pub name: String,
    pub seconds: f32,
    /// 0 to 1.
    pub progress: f32,
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

    /// Hands an animated prop back to its idle loop (see `Prop::idle`), dropping the frame it was left on.
    pub fn resume(&self, id: &str) {
        let mut data = self.0.borrow_mut();
        data.animations.remove(id);
        data.frames.remove(id);
    }

    /// The frame of a prop that loops its first `count` frames at `fps` (`phase` seconds ahead) while
    /// no script is playing or has set a frame for it; otherwise what `frame` says.
    pub fn idle_frame(&self, id: &str, count: usize, fps: f32, phase: f32) -> usize {
        let data = self.0.borrow();
        if count == 0 || data.animations.contains_key(id) || data.frames.contains_key(id) {
            drop(data);
            return self.frame(id);
        }
        (((data.clock + phase as f64) * fps as f64) as usize) % count
    }

    /// The car now passing the window, if any.
    pub fn car(&self) -> Option<Car> {
        self.0.borrow().street.car()
    }

    /// Restarts the traffic from a seed, so the cars come at different times.
    /// Sets how often cars come, from the room the player is in.
    pub fn set_street_timing(&self, timing: crate::street::Timing) {
        self.0.borrow_mut().street.set_timing(timing);
    }

    pub fn seed_street(&self, seed: u32) {
        self.0.borrow_mut().street.reseed(seed);
    }

    pub fn is_animating(&self, id: &str) -> bool {
        self.0.borrow().animations.contains_key(id)
    }

    /// Moves animations on; one that has shown its last frame ends.
    pub fn advance(&self, delta_seconds: f32) {
        let mut data = self.0.borrow_mut();
        data.clock += delta_seconds as f64;
        if let Some(effect) = &mut data.effect {
            effect.elapsed += delta_seconds;
            if effect.elapsed >= effect.seconds {
                data.effect = None;
            }
        }
        data.street.advance(delta_seconds);
        for animation in data.animations.values_mut() {
            animation.elapsed += delta_seconds;
        }
        data.animations.retain(|_, animation| !animation.finished());
    }
}

/// What a saved game keeps of the `World`: everything but animations in progress and the clock.
/// The lists are sorted so the same world always saves the same way.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldState {
    pub inventory: Vec<String>,
    pub hidden: Vec<String>,
    pub frames: Vec<(String, usize)>,
    pub lit: Vec<String>,
    pub flags: Vec<String>,
}

impl World {
    /// Sets a named flag scripts use to remember something (for example that the barman served a beer).
    pub fn set_flag(&self, name: &str, on: bool) {
        let mut data = self.0.borrow_mut();
        if on {
            data.flags.insert(name.to_owned());
        } else {
            data.flags.remove(name);
        }
    }

    pub fn flag(&self, name: &str) -> bool {
        self.0.borrow().flags.contains(name)
    }

    pub fn export(&self) -> WorldState {
        let data = self.0.borrow();
        let sorted = |set: &HashSet<String>| {
            let mut list: Vec<String> = set.iter().cloned().collect();
            list.sort();
            list
        };
        let mut frames: Vec<(String, usize)> = data
            .frames
            .iter()
            .map(|(id, frame)| (id.clone(), *frame))
            .collect();
        frames.sort();
        WorldState {
            inventory: data.inventory.clone(),
            hidden: sorted(&data.hidden),
            frames,
            lit: sorted(&data.lit),
            flags: sorted(&data.flags),
        }
    }

    /// Replaces everything `export` covers; animations in progress stop.
    pub fn import(&self, state: &WorldState) {
        let mut data = self.0.borrow_mut();
        data.inventory = state.inventory.clone();
        data.hidden = state.hidden.iter().cloned().collect();
        data.frames = state.frames.iter().cloned().collect();
        data.lit = state.lit.iter().cloned().collect();
        data.flags = state.flags.iter().cloned().collect();
        data.animations.clear();
    }
}

impl World {
    /// Starts a whole-screen effect (a shader named in the game's `shaders` folder) for `seconds`,
    /// replacing any running one. Not saved.
    pub fn start_screen_effect(&self, name: &str, seconds: f32) {
        self.0.borrow_mut().effect = Some(Effect {
            name: name.to_owned(),
            seconds,
            elapsed: 0.0,
        });
    }

    /// The effect now running, if any.
    pub fn screen_effect(&self) -> Option<ScreenEffect> {
        self.0.borrow().effect.as_ref().map(|effect| ScreenEffect {
            name: effect.name.clone(),
            seconds: effect.seconds,
            progress: (effect.elapsed / effect.seconds).clamp(0.0, 1.0),
        })
    }
}

#[cfg(test)]
mod effect_tests {
    use super::*;

    #[test]
    fn a_screen_effect_runs_for_its_time_then_ends() {
        let world = World::default();
        assert_eq!(world.screen_effect(), None);
        world.start_screen_effect("corruption", 0.5);
        world.advance(0.25);
        let effect = world.screen_effect().unwrap();
        assert_eq!(effect.name, "corruption");
        assert!((effect.progress - 0.5).abs() < 1e-4);
        world.advance(0.3);
        assert_eq!(world.screen_effect(), None);
    }
}
