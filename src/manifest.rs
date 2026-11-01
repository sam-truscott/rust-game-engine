//! `game.lua`, the file that makes a folder a game: where to start, which rooms exist, and the
//! few things the engine would otherwise have to hard-code.

use macroquad::color::Color;
use macroquad::math::Vec2;
use mlua::Table;

use crate::assets;

pub const MANIFEST: &str = "game.lua";

/// What the player says when a script has nothing to say about a click.
#[derive(Clone, Debug)]
pub struct Defaults {
    pub look: String,
    pub interact: String,
    pub use_with: String,
}

/// Timings and speeds. Missing keys keep the engine's own values.
#[derive(Clone, Copy, Debug)]
pub struct Tuning {
    pub walk_speed: f32,
    pub speech_seconds: f32,
    pub speech_seconds_per_char: f32,
    pub double_click_seconds: f32,
    pub double_click_distance: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            walk_speed: 60.0,
            speech_seconds: 1.0,
            speech_seconds_per_char: 0.05,
            double_click_seconds: 0.4,
            double_click_distance: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub panel: Color,
    /// Alpha of the cover over a dark room whose lights are off.
    pub darkness: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: Color::new(0.1, 0.1, 0.2, 1.0),
            panel: Color::new(0.06, 0.06, 0.1, 1.0),
            darkness: 0.99,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub id: String,
    pub title: String,
    pub start_room: String,
    pub start: Vec2,
    pub rooms: Vec<String>,
    pub intro: Option<String>,
    pub items: String,
    pub player: String,
    pub cursor: String,
    pub defaults: Defaults,
    pub tuning: Tuning,
    pub theme: Theme,
}

fn fail(message: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(message.into())
}

impl Manifest {
    pub fn load() -> Result<Self, String> {
        let source = assets::text(MANIFEST).ok_or_else(|| format!("missing {MANIFEST}"))?;
        Self::from_lua(&source, MANIFEST).map_err(|error| format!("{MANIFEST}: {error}"))
    }

    pub fn from_lua(source: &str, name: &str) -> mlua::Result<Self> {
        let lua = crate::script::data_lua()?;
        let table: Table = lua.load(source).set_name(name).eval()?;

        let required = |key: &str| -> mlua::Result<String> {
            table
                .get::<Option<String>>(key)?
                .ok_or_else(|| fail(format!("{key} is missing")))
        };
        let id = required("id")?;
        if id.is_empty() || id.contains(['/', '\\', ':', '.']) {
            return Err(fail("id must be a plain folder name"));
        }
        let rooms: Vec<String> = table
            .get::<Option<Vec<String>>>("rooms")?
            .filter(|rooms| !rooms.is_empty())
            .ok_or_else(|| fail("rooms needs at least one room id"))?;
        let start: Table = table
            .get::<Option<Table>>("start")?
            .ok_or_else(|| fail("start is missing"))?;
        let start_room = start
            .get::<Option<String>>("room")?
            .unwrap_or_else(|| rooms[0].clone());
        if !rooms.contains(&start_room) {
            return Err(fail(format!("start room {start_room} is not in rooms")));
        }
        let (x, y): (f32, f32) = (start.get("x")?, start.get("y")?);
        if !x.is_finite() || !y.is_finite() {
            return Err(fail("start needs finite x and y"));
        }

        let defaults: Option<Table> = table.get("defaults")?;
        let line = |key: &str, fallback: &str| -> mlua::Result<String> {
            let value = match &defaults {
                Some(t) => t.get::<Option<String>>(key)?,
                None => None,
            };
            Ok(value.unwrap_or_else(|| fallback.to_owned()))
        };
        let defaults = Defaults {
            look: line("look", "I see nothing special.")?,
            interact: line("interact", "I can't do that.")?,
            use_with: line("use", "That doesn't work.")?,
        };

        let mut tuning = Tuning::default();
        if let Some(t) = table.get::<Option<Table>>("tuning")? {
            let number = |key: &str, value: &mut f32| -> mlua::Result<()> {
                if let Some(v) = t.get::<Option<f32>>(key)? {
                    if !v.is_finite() || v <= 0.0 {
                        return Err(fail(format!("tuning.{key} must be positive")));
                    }
                    *value = v;
                }
                Ok(())
            };
            number("walk_speed", &mut tuning.walk_speed)?;
            number("speech_seconds", &mut tuning.speech_seconds)?;
            number(
                "speech_seconds_per_char",
                &mut tuning.speech_seconds_per_char,
            )?;
            number("double_click_seconds", &mut tuning.double_click_seconds)?;
            number("double_click_distance", &mut tuning.double_click_distance)?;
        }

        let mut theme = Theme::default();
        if let Some(t) = table.get::<Option<Table>>("theme")? {
            let colour = |key: &str, value: &mut Color| -> mlua::Result<()> {
                if let Some(rgb) = t.get::<Option<Table>>(key)? {
                    *value = Color::new(rgb.get(1)?, rgb.get(2)?, rgb.get(3)?, 1.0);
                }
                Ok(())
            };
            colour("background", &mut theme.background)?;
            colour("panel", &mut theme.panel)?;
            if let Some(d) = t.get::<Option<f32>>("darkness")? {
                theme.darkness = d.clamp(0.0, 1.0);
            }
        }

        Ok(Manifest {
            title: table
                .get::<Option<String>>("title")?
                .unwrap_or_else(|| id.clone()),
            id,
            start_room,
            start: Vec2::new(x, y),
            rooms,
            intro: table.get("intro")?,
            items: table
                .get::<Option<String>>("items")?
                .unwrap_or_else(|| "items.lua".to_owned()),
            player: required("player")?,
            cursor: required("cursor")?,
            defaults,
            tuning,
            theme,
        })
    }
}
