//! The game without the drawing: the room, the player, scripts, what is carried, and what a click does.

use macroquad::math::Vec2;

use crate::actor::Actor;
use crate::assets;
use crate::items::Items;
use crate::manifest::Manifest;
use crate::room::{Hotspot, Room};
use crate::save::SaveData;
use crate::script::ScriptRunner;
use crate::ui;
use crate::world::World;

/// Script handlers: `hotspot.<id>.look` and `.interact`, and the same for items.
const LOOK: &str = "look";
const INTERACT: &str = "interact";

pub fn load_room(id: &str) -> Result<Room, String> {
    let path = format!("rooms/{id}.lua");
    let source = assets::text(&path).ok_or_else(|| format!("missing {path}"))?;
    Room::from_lua(&source, &path).map_err(|error| format!("{path}: {error}"))
}

pub struct Session {
    pub manifest: Manifest,
    pub room: Room,
    /// The current room's id, such as "hall".
    pub room_id: String,
    pub player: Actor,
    pub scripts: ScriptRunner,
    pub world: World,
    pub items: Items,
    /// An item picked by double clicking it, waiting to be used on something.
    pub holding: Option<String>,
    pub cursor: Vec2,
    clock: f32,
    last_click: Option<(f32, Vec2)>,
    /// A carried item single clicked at this time, looked at once no second click has made it a
    /// double click; looking at once would show its line until the second click cut it short.
    pending_look: Option<(String, f32)>,
    /// Set when a script started that a click must not cut short (using an item, interacting).
    committed: bool,
    /// When the action running now was started by a click; a click this soon after is the second half of
    /// a double click, not one that skips its first line.
    started_at: f32,
    room_changed: bool,
}

impl Session {
    /// Loads every room's script and the items, and starts the intro in the first room.
    pub fn load() -> Result<Self, String> {
        let text = |path: &str| assets::text(path).ok_or_else(|| format!("missing {path}"));
        let failed = |path: &str, error: mlua::Error| format!("{path}: {error}");

        let manifest = Manifest::load()?;
        let room = load_room(&manifest.start_room)?;
        let items = Items::from_lua(&text(&manifest.items)?, &manifest.items)
            .map_err(|error| failed(&manifest.items, error))?;

        let world = World::default();
        let mut scripts =
            ScriptRunner::with_world(world.clone()).map_err(|error| failed("Lua", error))?;
        scripts
            .set_defaults(
                &manifest.defaults.look,
                &manifest.defaults.interact,
                &manifest.defaults.use_with,
            )
            .map_err(|error| failed("Lua", error))?;
        for id in &manifest.rooms {
            let path = format!("scripts/{id}.lua");
            let source = text(&path)?;
            scripts
                .set_room(id)
                .and_then(|()| scripts.load_definitions(&path, &source))
                .map_err(|error| failed(&path, error))?;
        }
        scripts
            .set_room(&manifest.start_room)
            .map_err(|error| failed("Lua", error))?;
        if let Some(intro) = &manifest.intro {
            scripts
                .start(intro, &text(intro)?)
                .map_err(|error| failed(intro, error))?;
        }

        if let Some(street) = room.street() {
            world.set_street_timing(street.timing);
        }
        let start = room.constrain(manifest.start.x, manifest.start.y);
        Ok(Session {
            room,
            room_id: manifest.start_room.clone(),
            player: Actor::with_tuning(start.x, start.y, manifest.tuning),
            manifest,
            scripts,
            world,
            items,
            holding: None,
            cursor: Vec2::ZERO,
            clock: 0.0,
            last_click: None,
            pending_look: None,
            committed: false,
            started_at: 0.0,
            room_changed: false,
        })
    }

    /// What a save keeps of this moment.
    pub fn capture(&self) -> SaveData {
        let position = self.player.position();
        SaveData {
            room: self.room_id.clone(),
            x: position.x,
            y: position.y,
            facing: self.player.facing(),
            world: self.world.export(),
        }
    }

    /// A session at the moment a save was made, without the intro. The player stands still and
    /// nothing is held.
    pub fn from_save(data: &SaveData) -> Result<Self, String> {
        let mut session = Self::load()?;
        if !session.manifest.rooms.contains(&data.room) {
            return Err(format!("save is in unknown room {}", data.room));
        }
        session.scripts.clear();
        session.world.import(&data.world);
        session.enter_room(&data.room, data.x, data.y);
        if session.room_id != data.room {
            return Err(format!("could not load room {}", data.room));
        }
        session.player.set_facing(data.facing);
        session.room_changed = false;
        Ok(session)
    }
    /// Whether the room is drawn dark: it is a dark room and its lights are off.
    pub fn is_dark(&self) -> bool {
        self.room.is_dark() && !self.world.is_lit(&self.room_id)
    }

    /// True once after the room has changed, so the drawing can load the new room's images.
    pub fn take_room_changed(&mut self) -> bool {
        std::mem::take(&mut self.room_changed)
    }

    fn enter_room(&mut self, id: &str, x: f32, y: f32) {
        let result = load_room(id).and_then(|room| {
            self.scripts
                .set_room(id)
                .map_err(|error| format!("{id}: {error}"))?;
            Ok(room)
        });
        match result {
            Ok(room) => {
                let spot = room.constrain(x, y);
                if let Some(street) = room.street() {
                    self.world.set_street_timing(street.timing);
                }
                self.room = room;
                self.room_id = id.to_owned();
                self.player = Actor::with_tuning(spot.x, spot.y, self.manifest.tuning);
                self.room_changed = true;
            }
            Err(error) => eprintln!("[room] {error}"),
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.clock += delta_seconds;
        if let Some((item, _)) = self
            .pending_look
            .take_if(|(_, at)| self.clock - *at > self.manifest.tuning.double_click_seconds)
            && !self.scripts.is_running()
        {
            self.started_at = self.clock;
            report(&item, self.scripts.interact_item(&item, LOOK));
        }
        self.player.update(delta_seconds);
        self.scripts
            .update(delta_seconds, &self.room, &mut self.player);
        if let Some(change) = self.scripts.take_room_change() {
            self.enter_room(&change.room, change.x, change.y);
        }
    }

    pub fn move_cursor(&mut self, point: Vec2) {
        self.cursor = point;
    }

    /// The hotspot under a point, ignoring ones scripts have hidden.
    pub fn hotspot_at(&self, point: Vec2) -> Option<&Hotspot> {
        self.room
            .hotspots()
            .iter()
            .rev()
            .find(|hotspot| hotspot.rect.contains(point) && !self.world.is_hidden(&hotspot.id))
    }

    fn item_at(&self, point: Vec2) -> Option<String> {
        let carried = self.world.items();
        ui::slot_at(point, carried.len()).map(|index| carried[index].clone())
    }

    /// Whether this click and the one before it are a double click; a double click is used up, so a
    /// third quick click starts a new one.
    fn is_double_click(&mut self, point: Vec2) -> bool {
        let tuning = self.manifest.tuning;
        let double = self.last_click.is_some_and(|(time, at)| {
            self.clock - time <= tuning.double_click_seconds
                && at.distance(point) <= tuning.double_click_distance
        });
        self.last_click = (!double).then_some((self.clock, point));
        double
    }

    /// A click at a screen point (see `ui`). A single click looks at what is under it, or walks if
    /// there is nothing; a double click interacts with it (through a door at once, or jumps to the spot if there is
    /// nothing). The single click has already started
    /// looking by the time the second arrives, so that is cancelled in favour of interacting. On a
    /// carried item there is no walk to hide the look, so it waits until a second click can no longer come.
    ///
    /// While an action (using an item, interacting) runs a click only skips the current speech line. While any
    /// other script runs (a look, the intro) clicks that would start something are ignored, except the second
    /// click of a double click, and skip the current line instead. With an item held (picked by double clicking it) the next click uses
    /// it on what is under it; that is never cut short, as a half-finished script could lose items.
    pub fn click(&mut self, point: Vec2) {
        if !self.scripts.is_running() {
            self.committed = false;
        }
        if self.input_locked() {
            // Remembered, so if this click ends the action the next one can still make it a double click.
            self.is_double_click(point);
            self.skip_line();
            return;
        }
        // Whatever this click does replaces the look a single click on an item is waiting to start.
        self.pending_look = None;
        let holding = self.holding.is_some();
        let double = self.is_double_click(point) && !holding;
        let busy = self.scripts.is_running();
        if ui::in_panel(point) {
            match self.item_at(point) {
                Some(item) => self.click_item(&item, double, busy),
                None if busy => self.skip_line(),
                None => {}
            }
        } else {
            self.click_room(ui::to_room(point), double, busy);
        }
        // Using a held item must not be cut short by a second quick click on the same target.
        if holding {
            self.last_click = None;
        }
    }

    /// Plays the running action at speed until it changes the room, up to a minute of game time;
    /// anything else it does on the way (walking, speech) happens as usual. Without a room change
    /// the action is left running where it got to.
    fn fast_forward_to_room_change(&mut self) {
        self.room_changed = false;
        for _ in 0..1200 {
            self.player.stop_speaking();
            self.update(0.05);
            if self.room_changed || !self.scripts.is_running() {
                break;
            }
        }
    }

    /// A click that starts nothing while a script runs skips the line being spoken; the script goes on
    /// to its next line or ends. Not for a click that is part of starting the action (a double click).
    fn skip_line(&mut self) {
        if self.clock - self.started_at > self.manifest.tuning.double_click_seconds {
            self.player.stop_speaking();
        }
    }

    fn commit(&mut self) {
        self.committed = true;
        self.started_at = self.clock;
    }

    /// Whether an action is running that ignores all clicks; the cursor is drawn faded meanwhile.
    pub fn input_locked(&self) -> bool {
        self.committed && self.scripts.is_running()
    }

    /// Stops what the player is doing so something else can start.
    fn interrupt(&mut self) {
        self.scripts.clear();
        self.player.stop_speaking();
    }

    fn click_item(&mut self, item: &str, double: bool, busy: bool) {
        if let Some(held) = self.holding.clone() {
            if busy {
                return;
            }
            self.holding = None;
            self.commit();
            let started = if held == item {
                self.scripts.interact_item(item, INTERACT)
            } else {
                self.scripts.use_with_item(&held, item)
            };
            report(item, started);
        } else if double && self.items.get(item).is_some_and(|i| i.self_use) {
            self.interrupt();
            self.commit();
            report(item, self.scripts.interact_item(item, INTERACT));
        } else if double {
            self.interrupt();
            self.holding = Some(item.to_owned());
        } else if !busy {
            self.pending_look = Some((item.to_owned(), self.clock));
        } else {
            self.skip_line();
        }
    }

    fn click_room(&mut self, point: Vec2, double: bool, busy: bool) {
        let hotspot = self.hotspot_at(point).cloned();
        if let Some(held) = self.holding.clone() {
            if busy {
                return;
            }
            self.holding = None;
            self.commit();
            if let Some(hotspot) = hotspot {
                let walk_to = hotspot.walk_to.or(Some(point));
                report(
                    &hotspot.id,
                    self.scripts.use_with_hotspot(&held, &hotspot.id, walk_to),
                );
            }
            return;
        }
        match hotspot {
            Some(hotspot) if double || !busy => {
                let verb = if double { INTERACT } else { LOOK };
                if double {
                    self.interrupt();
                    self.commit();
                } else {
                    self.started_at = self.clock;
                }
                let walk_to = hotspot.walk_to.or(Some(point));
                report(
                    &hotspot.id,
                    self.scripts.interact(&hotspot.id, verb, walk_to),
                );
                if double && hotspot.door {
                    self.fast_forward_to_room_change();
                }
            }
            Some(_) => self.skip_line(),
            None if double && !busy => {
                self.player.teleport(&self.room, point.x, point.y);
            }
            None if !busy => self.player.walk_to(&self.room, point.x, point.y),
            None => self.skip_line(),
        }
    }
}

fn report(what: &str, result: mlua::Result<()>) {
    if let Err(error) = result {
        eprintln!("[lua] {what}: {error}");
    }
}
