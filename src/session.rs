//! The game without the drawing: the room, the player, scripts, what is carried, and what a click does.

use macroquad::math::Vec2;

use crate::actor::Actor;
use crate::assets;
use crate::items::Items;
use crate::room::{Hotspot, Room};
use crate::script::ScriptRunner;
use crate::ui::{self, Verb};
use crate::world::World;

const START_ROOM: &str = "rooms/hall.lua";
const ROOM_SCRIPT: &str = "scripts/hall.lua";
const INTRO_SCRIPT: &str = "scripts/intro.lua";
const ITEMS: &str = "items.lua";
const START_POSITION: Vec2 = Vec2::new(60.0, 50.0);

pub struct Session {
    pub room: Room,
    pub player: Actor,
    pub scripts: ScriptRunner,
    pub world: World,
    pub items: Items,
    pub verb: Verb,
    /// An item picked with "Use", waiting to be used on something.
    pub holding: Option<String>,
    pub cursor: Vec2,
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
            verb: Verb::WalkTo,
            holding: None,
            cursor: Vec2::ZERO,
        })
    }

    pub fn update(&mut self, delta_seconds: f32) {
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

    /// A click at a screen point (see `ui`): on the panel it picks a verb or an item, above it it acts
    /// on the room. Choosing things (a verb, an item to use) always works, so a click is never lost
    /// while the player is still walking or talking; clicks that would start something wait until it
    /// is over.
    pub fn click(&mut self, point: Vec2) {
        let busy = self.scripts.is_running();
        if let Some(verb) = ui::verb_at(point) {
            self.verb = verb;
            self.holding = None;
            return;
        }
        if ui::in_panel(point) {
            if let Some(item) = self.item_at(point) {
                if !busy {
                    self.click_item(&item);
                } else if self.verb == Verb::Use && self.holding.is_none() {
                    self.holding = Some(item);
                }
            }
            return;
        }
        if !busy {
            self.click_room(ui::to_room(point));
        }
    }

    /// After something is done the verb goes back to "Walk to" and nothing is held.
    fn finish(&mut self) -> Verb {
        self.holding = None;
        std::mem::replace(&mut self.verb, Verb::WalkTo)
    }

    fn click_item(&mut self, item: &str) {
        if let Some(held) = self.holding.clone() {
            self.finish();
            let started = if held == item {
                self.scripts.interact_item(item, Verb::Use.key())
            } else {
                self.scripts.use_with_item(&held, item)
            };
            report(item, started);
        } else if self.verb == Verb::Use {
            self.holding = Some(item.to_owned());
        } else {
            let verb = carried_verb(self.finish());
            report(item, self.scripts.interact_item(item, verb.key()));
        }
    }

    fn click_room(&mut self, point: Vec2) {
        let held = self.holding.clone();
        let verb = self.finish();
        match (self.hotspot_at(point).cloned(), held) {
            (Some(hotspot), Some(held)) => {
                let walk_to = hotspot.walk_to.or(Some(point));
                report(
                    &hotspot.id,
                    self.scripts.use_with_hotspot(&held, &hotspot.id, walk_to),
                );
            }
            // "Walk to" on an object just walks towards where you clicked, so you can still walk
            // behind things; the standing spot is for the other verbs.
            (Some(hotspot), None) if verb != Verb::WalkTo => {
                let walk_to = hotspot.walk_to.or(Some(point));
                report(
                    &hotspot.id,
                    self.scripts.interact(&hotspot.id, verb.key(), walk_to),
                );
            }
            (None, Some(_)) => {}
            (_, None) => self.player.walk_to(&self.room, point.x, point.y),
        }
    }
}

/// You cannot walk to something in your pocket, so the default verb on a carried item is "Look at".
fn carried_verb(verb: Verb) -> Verb {
    if verb == Verb::WalkTo {
        Verb::LookAt
    } else {
        verb
    }
}

fn report(what: &str, result: mlua::Result<()>) {
    if let Err(error) = result {
        eprintln!("[lua] {what}: {error}");
    }
}
