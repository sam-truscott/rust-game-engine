//! Lua scripts run as coroutines. A blocking call such as `walkTo` records a command and yields;
//! the runner applies the command and resumes the script once what it started has finished.

use std::cell::RefCell;
use std::rc::Rc;

use macroquad::math::Vec2;
use mlua::{Function, Lua, Thread};

use crate::actor::Actor;
use crate::room::{DEFAULT_FPS, Room};
use crate::world::World;

/// The blocking calls are plain Lua wrappers: the Rust function only records what was asked for.
const PRELUDE: &str = r#"
function walkTo(x, y) __walk_to(x, y) coroutine.yield() end
function say(text) __say(text) coroutine.yield() end
function wait(seconds) __wait(seconds) coroutine.yield() end
-- Plays frames first..last (0-based, either direction) of an animated prop, then stays on the last.
function animate(id, first, last, fps) __animate(id, first, last, fps) coroutine.yield() end
-- Leaves for another room (its id, such as "closet"), arriving at x, y. The script carries on once there.
function enterRoom(room, x, y) __enter_room(room, x, y) coroutine.yield() end

-- Scripts fill these in: hotspot.counter = { look = function() say("...") end, interact = ... } and
-- item.brass_key = { look = ... }. A single click runs look, a double click interact. A table may also
-- hold use_with = { other_item = function() ... end }.
hotspot = {}
item = {}

-- Each room has its own table of hotspot handlers, so ids only have to be unique within a room.
__hotspots = {}
function __scope(room)
  __hotspots[room] = __hotspots[room] or {}
  hotspot = __hotspots[room]
end

-- What happens when look or interact is used on an item the player carries.
function __item_interaction(id, verb)
  return function()
    local handlers = item[id]
    local handler = handlers and handlers[verb]
    if handler then
      handler()
    elseif verb == "look" then
      say("I see nothing special.")
    else
      say("I can't do that.")
    end
  end
end

-- Using the carried item `held` on a hotspot (walking to x, y first) or on another carried item.
function __use_with(held, kind, target, x, y)
  return function()
    if x then walkTo(x, y) end
    local owner = (kind == "item") and item[target] or hotspot[target]
    local handler = owner and owner.use_with and owner.use_with[held]
    if handler then
      handler()
    else
      say("That doesn't work.")
    end
  end
end

-- What happens when look or interact is used on a hotspot: walk to its spot, then run the handler if there is one.
function __interaction(id, verb, x, y)
  return function()
    if x then walkTo(x, y) end
    local handlers = hotspot[id]
    local handler = handlers and handlers[verb]
    if handler then
      handler()
    elseif verb == "look" then
      say("I see nothing special.")
    else
      say("I can't do that.")
    end
  end
end
"#;

enum Command {
    WalkTo(f32, f32),
    Say(String),
    Wait(f32),
    Animate(String, usize, usize, Option<f32>),
    EnterRoom(String, f32, f32),
}

enum Waiting {
    Walking,
    Speaking,
    Seconds(f32),
    Animating(String),
}

impl Waiting {
    fn finished(&mut self, delta_seconds: f32, actor: &Actor, world: &World) -> bool {
        match self {
            Waiting::Walking => !actor.is_walking(),
            Waiting::Speaking => !actor.is_speaking(),
            Waiting::Seconds(remaining) => {
                *remaining -= delta_seconds;
                *remaining <= 0.0
            }
            Waiting::Animating(id) => !world.is_animating(id),
        }
    }
}

struct Script {
    name: String,
    thread: Thread,
    waiting: Option<Waiting>,
}

pub struct ScriptRunner {
    lua: Lua,
    scripts: Vec<Script>,
    pending: Rc<RefCell<Option<Command>>>,
    world: World,
    room_change: Option<RoomChange>,
}

/// A script asked to move to another room; `Session` carries it out.
pub struct RoomChange {
    pub room: String,
    pub x: f32,
    pub y: f32,
}

impl ScriptRunner {
    pub fn new() -> mlua::Result<Self> {
        Self::with_world(World::default())
    }

    /// `give`, `take`, `has`, `hide`, `setFrame`, `setLight` and `isLit` in scripts act on this world.
    pub fn with_world(world: World) -> mlua::Result<Self> {
        let lua = Lua::new();
        let pending = Rc::new(RefCell::new(None));

        let slot = pending.clone();
        let walk_to = lua.create_function(move |_, (x, y): (f32, f32)| {
            *slot.borrow_mut() = Some(Command::WalkTo(x, y));
            Ok(())
        })?;
        let slot = pending.clone();
        let say = lua.create_function(move |_, text: String| {
            *slot.borrow_mut() = Some(Command::Say(text));
            Ok(())
        })?;
        let slot = pending.clone();
        let wait = lua.create_function(move |_, seconds: f32| {
            *slot.borrow_mut() = Some(Command::Wait(seconds));
            Ok(())
        })?;

        let slot = pending.clone();
        let animate = lua.create_function(
            move |_, (id, first, last, fps): (String, usize, usize, Option<f32>)| {
                *slot.borrow_mut() = Some(Command::Animate(id, first, last, fps));
                Ok(())
            },
        )?;

        let slot = pending.clone();
        let enter_room = lua.create_function(move |_, (room, x, y): (String, f32, f32)| {
            *slot.borrow_mut() = Some(Command::EnterRoom(room, x, y));
            Ok(())
        })?;

        let globals = lua.globals();
        globals.set("__enter_room", enter_room)?;
        globals.set("__animate", animate)?;
        globals.set("__walk_to", walk_to)?;
        globals.set("__say", say)?;
        globals.set("__wait", wait)?;

        let state = world.clone();
        globals.set(
            "give",
            lua.create_function(move |_, id: String| {
                state.give(&id);
                Ok(())
            })?,
        )?;
        let state = world.clone();
        globals.set(
            "take",
            lua.create_function(move |_, id: String| Ok(state.take(&id)))?,
        )?;
        let state = world.clone();
        globals.set(
            "has",
            lua.create_function(move |_, id: String| Ok(state.has(&id)))?,
        )?;
        let state = world.clone();
        globals.set(
            "hide",
            lua.create_function(move |_, id: String| {
                state.hide(&id);
                Ok(())
            })?,
        )?;
        let state = world.clone();
        globals.set(
            "setLight",
            lua.create_function(move |_, (room, on): (String, bool)| {
                state.set_light(&room, on);
                Ok(())
            })?,
        )?;
        let state = world.clone();
        globals.set(
            "isLit",
            lua.create_function(move |_, room: String| Ok(state.is_lit(&room)))?,
        )?;
        let state = world.clone();
        globals.set(
            "setFrame",
            lua.create_function(move |_, (id, frame): (String, usize)| {
                state.set_frame(&id, frame);
                Ok(())
            })?,
        )?;
        lua.load(PRELUDE).set_name("prelude").exec()?;

        Ok(ScriptRunner {
            lua,
            scripts: Vec::new(),
            pending,
            world,
            room_change: None,
        })
    }

    /// Makes the hotspot handlers a script defines belong to this room (until the next call).
    pub fn set_room(&self, room: &str) -> mlua::Result<()> {
        let scope: Function = self.lua.globals().get("__scope")?;
        scope.call(room)
    }

    /// The room change a script has asked for, if any.
    pub fn take_room_change(&mut self) -> Option<RoomChange> {
        self.room_change.take()
    }

    pub fn world(&self) -> World {
        self.world.clone()
    }

    pub fn start(&mut self, name: &str, source: &str) -> mlua::Result<()> {
        let function = self.lua.load(source).set_name(name).into_function()?;
        let thread = self.lua.create_thread(function)?;
        self.scripts.push(Script {
            name: name.to_owned(),
            thread,
            waiting: None,
        });
        Ok(())
    }

    /// Runs a script to completion straight away; for scripts that only define things, such as hotspot
    /// handlers. It must not call anything that waits.
    pub fn load_definitions(&mut self, name: &str, source: &str) -> mlua::Result<()> {
        self.lua.load(source).set_name(name).exec()
    }

    /// Starts `verb` (`look` or `interact`) on a hotspot, walking to `walk_to` first.
    pub fn interact(&mut self, id: &str, verb: &str, walk_to: Option<Vec2>) -> mlua::Result<()> {
        let (x, y) = (walk_to.map(|p| p.x), walk_to.map(|p| p.y));
        self.spawn("__interaction", (id, verb, x, y), format!("{id}.{verb}"))
    }

    /// Starts `verb` (`look` or `interact`) on an item the player carries.
    pub fn interact_item(&mut self, id: &str, verb: &str) -> mlua::Result<()> {
        self.spawn("__item_interaction", (id, verb), format!("{id}.{verb}"))
    }

    /// Starts using the carried item `held` on a hotspot, walking to `walk_to` first.
    pub fn use_with_hotspot(
        &mut self,
        held: &str,
        id: &str,
        walk_to: Option<Vec2>,
    ) -> mlua::Result<()> {
        let (x, y) = (walk_to.map(|p| p.x), walk_to.map(|p| p.y));
        self.spawn(
            "__use_with",
            (held, "hotspot", id, x, y),
            format!("{held}+{id}"),
        )
    }

    /// Starts using the carried item `held` on another carried item.
    pub fn use_with_item(&mut self, held: &str, id: &str) -> mlua::Result<()> {
        let none: Option<f32> = None;
        self.spawn(
            "__use_with",
            (held, "item", id, none, none),
            format!("{held}+{id}"),
        )
    }

    /// Starts a script made by calling a prelude function that returns the function to run.
    fn spawn(
        &mut self,
        factory: &str,
        args: impl mlua::IntoLuaMulti,
        name: String,
    ) -> mlua::Result<()> {
        let factory: Function = self.lua.globals().get(factory)?;
        let function: Function = factory.call(args)?;
        let thread = self.lua.create_thread(function)?;
        self.scripts.push(Script {
            name,
            thread,
            waiting: None,
        });
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        !self.scripts.is_empty()
    }

    /// Ends all scripts. A suspended coroutine is just dropped; there is no thread to release.
    pub fn clear(&mut self) {
        self.scripts.clear();
    }

    pub fn update(&mut self, delta_seconds: f32, room: &Room, actor: &mut Actor) {
        self.world.advance(delta_seconds);
        let mut i = 0;
        while i < self.scripts.len() {
            let script = &mut self.scripts[i];
            if let Some(waiting) = &mut script.waiting {
                if !waiting.finished(delta_seconds, actor, &self.world) {
                    i += 1;
                    continue;
                }
                script.waiting = None;
            }

            match script.thread.resume::<()>(()) {
                Err(error) => {
                    eprintln!("[lua] {}: {error}", script.name);
                    self.scripts.remove(i);
                }
                Ok(()) if script.thread.is_resumable() => {
                    script.waiting =
                        self.pending
                            .borrow_mut()
                            .take()
                            .map(|command| match command {
                                Command::WalkTo(x, y) => {
                                    actor.walk_to(room, x, y);
                                    Waiting::Walking
                                }
                                Command::Say(text) => {
                                    actor.say(&text);
                                    Waiting::Speaking
                                }
                                Command::Wait(seconds) => Waiting::Seconds(seconds),
                                Command::EnterRoom(room, x, y) => {
                                    self.room_change = Some(RoomChange { room, x, y });
                                    Waiting::Seconds(0.0)
                                }
                                Command::Animate(id, first, last, fps) => {
                                    let fps = fps.unwrap_or_else(|| {
                                        room.props()
                                            .iter()
                                            .find(|prop| prop.id.as_deref() == Some(id.as_str()))
                                            .map_or(DEFAULT_FPS, |prop| prop.fps)
                                    });
                                    self.world.animate(&id, first, last, fps);
                                    Waiting::Animating(id)
                                }
                            });
                    i += 1;
                }
                Ok(()) => {
                    println!("[lua] {} finished", script.name);
                    self.scripts.remove(i);
                }
            }
        }
    }
}
