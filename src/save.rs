//! Saved games: what is kept (`SaveData`), its text format, and the folder the saves live in.

use std::path::{Path, PathBuf};

use crate::direction::Direction;
use crate::world::WorldState;

const HEADER: &str = "game-save 1";
pub const AUTOSAVE: &str = "autosave";
const NUMBERED: &str = "save-";
const EXTENSION: &str = "sav";

/// Where the player is and what has changed in the world. Scripts mid-way are not kept: a loaded game
/// starts with the player standing still.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveData {
    pub room: String,
    pub x: f32,
    pub y: f32,
    pub facing: Direction,
    pub world: WorldState,
}

fn facing_name(facing: Direction) -> &'static str {
    match facing {
        Direction::Front => "front",
        Direction::Back => "back",
        Direction::Left => "left",
        Direction::Right => "right",
    }
}

fn parse_facing(name: &str) -> Option<Direction> {
    [
        Direction::Front,
        Direction::Back,
        Direction::Left,
        Direction::Right,
    ]
    .into_iter()
    .find(|&facing| facing_name(facing) == name)
}

impl SaveData {
    pub fn to_text(&self) -> String {
        let mut text = format!(
            "{HEADER}\nroom {}\nposition {} {}\nfacing {}\n",
            self.room,
            self.x,
            self.y,
            facing_name(self.facing)
        );
        let world = &self.world;
        let lists = [
            ("item", &world.inventory),
            ("hidden", &world.hidden),
            ("lit", &world.lit),
            ("flag", &world.flags),
        ];
        for (key, values) in lists {
            for value in values {
                text.push_str(&format!("{key} {value}\n"));
            }
        }
        for (id, frame) in &world.frames {
            text.push_str(&format!("frame {id} {frame}\n"));
        }
        text
    }

    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next().map(str::trim) != Some(HEADER) {
            return Err("not a save file of this version".to_owned());
        }
        let mut room = None;
        let mut position = None;
        let mut facing = Direction::Front;
        let mut world = WorldState::default();
        for line in lines {
            let words: Vec<&str> = line.split_whitespace().collect();
            let bad = || format!("bad line: {line}");
            match words.as_slice() {
                [] => {}
                ["room", id] => room = Some((*id).to_owned()),
                ["position", x, y] => match (x.parse::<f32>(), y.parse::<f32>()) {
                    (Ok(x), Ok(y)) if x.is_finite() && y.is_finite() => position = Some((x, y)),
                    _ => return Err(bad()),
                },
                ["facing", name] => facing = parse_facing(name).ok_or_else(bad)?,
                ["item", id] => world.inventory.push((*id).to_owned()),
                ["hidden", id] => world.hidden.push((*id).to_owned()),
                ["lit", id] => world.lit.push((*id).to_owned()),
                ["flag", id] => world.flags.push((*id).to_owned()),
                ["frame", id, frame] => {
                    let frame = frame.parse().map_err(|_| bad())?;
                    world.frames.push(((*id).to_owned(), frame));
                }
                _ => return Err(bad()),
            }
        }
        Ok(SaveData {
            room: room.ok_or("no room")?,
            x: position.ok_or("no position")?.0,
            y: position.ok_or("no position")?.1,
            facing,
            world,
        })
    }
}

/// One save in the folder, as the load menu lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveEntry {
    /// The file name without its extension: "autosave" or "save-<number>".
    pub name: String,
    /// What the menu shows: "Autosave" or "Save <number>".
    pub label: String,
}

fn number_of(name: &str) -> Option<u32> {
    name.strip_prefix(NUMBERED)?.parse().ok()
}

fn is_valid_name(name: &str) -> bool {
    name == AUTOSAVE || number_of(name).is_some()
}

/// A folder of saves: one autosave, which closing the game overwrites, and any number of numbered saves.
pub struct SaveStore {
    dir: PathBuf,
}

impl SaveStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        SaveStore { dir: dir.into() }
    }

    /// `GAME_SAVE_DIR` if set, else `%APPDATA%\Game\saves` (`~/.local/share/game/saves` elsewhere),
    /// else `saves` in the working directory.
    pub fn in_default_dir() -> Self {
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        let dir = var("GAME_SAVE_DIR")
            .or_else(|| var("APPDATA").map(|dir| dir.join("Game").join("saves")))
            .or_else(|| var("HOME").map(|dir| dir.join(".local/share/game/saves")))
            .unwrap_or_else(|| PathBuf::from("saves"));
        SaveStore::new(dir)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.{EXTENSION}"))
    }

    /// The autosave first, then numbered saves, highest number first.
    pub fn list(&self) -> Vec<SaveEntry> {
        let mut numbers = Vec::new();
        let mut autosave = false;
        for entry in std::fs::read_dir(&self.dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some(EXTENSION) {
                continue;
            }
            match path.file_stem().and_then(|s| s.to_str()) {
                Some(AUTOSAVE) => autosave = true,
                Some(stem) => numbers.extend(number_of(stem)),
                None => {}
            }
        }
        numbers.sort_unstable_by(|a, b| b.cmp(a));
        let mut entries = Vec::new();
        if autosave {
            entries.push(SaveEntry {
                name: AUTOSAVE.to_owned(),
                label: "Autosave".to_owned(),
            });
        }
        entries.extend(numbers.into_iter().map(|n| SaveEntry {
            name: format!("{NUMBERED}{n}"),
            label: format!("Save {n}"),
        }));
        entries
    }

    pub fn read(&self, name: &str) -> Result<SaveData, String> {
        if !is_valid_name(name) {
            return Err(format!("not a save name: {name}"));
        }
        let path = self.path(name);
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        SaveData::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Writes to a temporary file first so a crash never leaves half a save.
    fn write(&self, name: &str, data: &SaveData) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let path = self.path(name);
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, data.to_text()).map_err(|e| format!("{}: {e}", temp.display()))?;
        std::fs::rename(&temp, &path).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn write_autosave(&self, data: &SaveData) -> Result<(), String> {
        self.write(AUTOSAVE, data)
    }

    fn next_number(&self) -> u32 {
        self.list()
            .iter()
            .filter_map(|entry| number_of(&entry.name))
            .max()
            .map_or(1, |highest| highest + 1)
    }

    /// Saves under the next free number and returns the new save's name.
    pub fn write_new(&self, data: &SaveData) -> Result<String, String> {
        let name = format!("{NUMBERED}{}", self.next_number());
        self.write(&name, data)?;
        Ok(name)
    }

    /// Turns the autosave into a numbered save, so starting a new game does not overwrite it.
    pub fn archive_autosave(&self) -> Result<(), String> {
        let from = self.path(AUTOSAVE);
        if !from.exists() {
            return Ok(());
        }
        let to = self.path(&format!("{NUMBERED}{}", self.next_number()));
        std::fs::rename(&from, &to).map_err(|e| format!("{}: {e}", from.display()))
    }
}
