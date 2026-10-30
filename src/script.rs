//! Lua scripts run as coroutines. A blocking call such as `walkTo` records a command and yields;
//! the runner applies the command and resumes the script once what it started has finished.

use std::cell::RefCell;
use std::rc::Rc;

use macroquad::math::Vec2;
use mlua::{Function, Lua, Thread};

use crate::actor::Actor;
use crate::room::Room;

/// The blocking calls are plain Lua wrappers: the Rust function only records what was asked for.
const PRELUDE: &str = r#"
function walkTo(x, y) __walk_to(x, y) coroutine.yield() end
function say(text) __say(text) coroutine.yield() end
function wait(seconds) __wait(seconds) coroutine.yield() end

-- Scripts fill this in: hotspot.counter = { look = function() say("...") end, ... }
hotspot = {}

-- What happens when a verb is used on a hotspot: walk to its spot, then run the handler if there is one.
function __interaction(id, verb, x, y)
  return function()
    if x then walkTo(x, y) end
    if verb == "walk" then return end
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
}

enum Waiting {
    Walking,
    Speaking,
    Seconds(f32),
}

impl Waiting {
    fn finished(&mut self, delta_seconds: f32, actor: &Actor) -> bool {
        match self {
            Waiting::Walking => !actor.is_walking(),
            Waiting::Speaking => !actor.is_speaking(),
            Waiting::Seconds(remaining) => {
                *remaining -= delta_seconds;
                *remaining <= 0.0
            }
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
}

impl ScriptRunner {
    pub fn new() -> mlua::Result<Self> {
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

        let globals = lua.globals();
        globals.set("__walk_to", walk_to)?;
        globals.set("__say", say)?;
        globals.set("__wait", wait)?;
        lua.load(PRELUDE).set_name("prelude").exec()?;

        Ok(ScriptRunner {
            lua,
            scripts: Vec::new(),
            pending,
        })
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

    /// Starts using `verb` (a key from `ui::Verb::key`) on a hotspot, walking to `walk_to` first.
    pub fn interact(&mut self, id: &str, verb: &str, walk_to: Option<Vec2>) -> mlua::Result<()> {
        let factory: Function = self.lua.globals().get("__interaction")?;
        let (x, y) = (walk_to.map(|p| p.x), walk_to.map(|p| p.y));
        let function: Function = factory.call((id, verb, x, y))?;
        let thread = self.lua.create_thread(function)?;
        self.scripts.push(Script {
            name: format!("{id}.{verb}"),
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
        let mut i = 0;
        while i < self.scripts.len() {
            let script = &mut self.scripts[i];
            if let Some(waiting) = &mut script.waiting {
                if !waiting.finished(delta_seconds, actor) {
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
