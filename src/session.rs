//! The game without the drawing: the room, the player, scripts, what is carried, and what a click does.

use macroquad::math::Vec2;

use crate::actor::Actor;
use crate::assets;
use crate::items::Items;
use crate::room::{Hotspot, Room};
use crate::script::ScriptRunner;
use crate::ui;
use crate::world::World;

const START_ROOM: &str = "rooms/hall.lua";
const ROOM_SCRIPT: &str = "scripts/hall.lua";
const INTRO_SCRIPT: &str = "scripts/intro.lua";
const ITEMS: &str = "items.lua";
const START_POSITION: Vec2 = Vec2::new(30.0, 50.0);

/// Script handlers: `hotspot.<id>.look` and `.interact`, and the same for items.
const LOOK: &str = "look";
const INTERACT: &str = "interact";
const DOUBLE_CLICK_SECONDS: f32 = 0.4;
const DOUBLE_CLICK_DISTANCE: f32 = 4.0;

pub struct Session {
    pub room: Room,
    pub player: Actor,
    pub scripts: ScriptRunner,
    pub world: World,
    pub items: Items,
    /// An item picked by double clicking it, waiting to be used on something.
    pub holding: Option<String>,
    pub cursor: Vec2,
    clock: f32,
    last_click: Option<(f32, Vec2)>,
}

impl Session {
    /// Loads the first room, its scripts and the items, and starts the intro.
    pub fn load() -> Result<Self, String> {
        let text = |path: &str| assets::text(path).ok_or_else(|| format!("missing {path}"));
        let failed = |path: &str, error: mlua::Error| format!("{path}: {error}");

        let room = Room::from_lua(&text(START_ROOM)?, START_ROOM)
            .map_err(|error| failed(START_ROOM, error))?;
        let items = Items::from_lua(&text(ITEMS)?, ITEMS).map_err(|error| failed(ITEMS, error))?;

        let world = World::default();
        let mut scripts =
            ScriptRunner::with_world(world.clone()).map_err(|error| failed("Lua", error))?;
        scripts
            .load_definitions(ROOM_SCRIPT, &text(ROOM_SCRIPT)?)
            .map_err(|error| failed(ROOM_SCRIPT, error))?;
        scripts
            .start(INTRO_SCRIPT, &text(INTRO_SCRIPT)?)
            .map_err(|error| failed(INTRO_SCRIPT, error))?;

        let start = room.constrain(START_POSITION.x, START_POSITION.y);
        Ok(Session {
            room,
            player: Actor::new(start.x, start.y),
            scripts,
            world,
            items,
            holding: None,
            cursor: Vec2::ZERO,
            clock: 0.0,
            last_click: None,
        })
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.clock += delta_seconds;
        self.player.update(delta_seconds);
        self.scripts
            .update(delta_seconds, &self.room, &mut self.player);
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
        let double = self.last_click.is_some_and(|(time, at)| {
            self.clock - time <= DOUBLE_CLICK_SECONDS && at.distance(point) <= DOUBLE_CLICK_DISTANCE
        });
        self.last_click = (!double).then_some((self.clock, point));
        double
    }

    /// A click at a screen point (see `ui`). A single click looks at what is under it, or walks if
    /// there is nothing; a double click interacts with it. The single click has already started
    /// looking by the time the second arrives, so that is cancelled in favour of interacting.
    ///
    /// Clicks that would start something are ignored while a script is running, except the second
    /// click of a double click. With an item held (picked by double clicking it) the next click uses
    /// it on what is under it; that is never cut short, as a half-finished script could lose items.
    pub fn click(&mut self, point: Vec2) {
        let double = self.is_double_click(point) && self.holding.is_none();
        let busy = self.scripts.is_running();
        if ui::in_panel(point) {
            if let Some(item) = self.item_at(point) {
                self.click_item(&item, double, busy);
            }
        } else {
            self.click_room(ui::to_room(point), double, busy);
        }
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
            let started = if held == item {
                self.scripts.interact_item(item, INTERACT)
            } else {
                self.scripts.use_with_item(&held, item)
            };
            report(item, started);
        } else if double {
            self.interrupt();
            self.holding = Some(item.to_owned());
        } else if !busy {
            report(item, self.scripts.interact_item(item, LOOK));
        }
    }

    fn click_room(&mut self, point: Vec2, double: bool, busy: bool) {
        let hotspot = self.hotspot_at(point).cloned();
        if let Some(held) = self.holding.clone() {
            if busy {
                return;
            }
            self.holding = None;
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
                }
                let walk_to = hotspot.walk_to.or(Some(point));
                report(
                    &hotspot.id,
                    self.scripts.interact(&hotspot.id, verb, walk_to),
                );
            }
            Some(_) => {}
            None if !busy => self.player.walk_to(&self.room, point.x, point.y),
            None => {}
        }
    }
}

fn report(what: &str, result: mlua::Result<()>) {
    if let Err(error) = result {
        eprintln!("[lua] {what}: {error}");
    }
}
