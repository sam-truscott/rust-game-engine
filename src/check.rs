//! Looks over the game the assets layer is pointing at and lists what is wrong with it, for
//! `game --check [dir]` and for tests that want to know a game is whole.

use std::collections::HashSet;

use crate::assets;
use crate::items::Items;
use crate::manifest::Manifest;
use crate::room::Room;
use crate::script::ScriptRunner;

/// Every problem found, empty when the game is sound. Loading problems stop the check early since
/// nothing after them can be looked at.
pub fn check_game() -> Vec<String> {
    let mut problems = Vec::new();
    let manifest = match Manifest::load() {
        Ok(manifest) => manifest,
        Err(error) => return vec![error],
    };

    for (what, path) in [("player", &manifest.player), ("cursor", &manifest.cursor)] {
        if assets::bytes(path).is_none() {
            problems.push(format!("{what} image {path} is missing"));
        }
    }
    match assets::text(&manifest.items) {
        None => problems.push(format!("missing {}", manifest.items)),
        Some(source) => {
            if let Err(error) = Items::from_lua(&source, &manifest.items) {
                problems.push(format!("{}: {error}", manifest.items));
            }
        }
    }
    if let Some(intro) = &manifest.intro
        && assets::text(intro).is_none()
    {
        problems.push(format!("missing {intro}"));
    }

    let mut prop_ids = HashSet::new();
    for id in &manifest.rooms {
        check_room(id, &mut prop_ids, &mut problems);
    }
    problems
}

fn check_room(id: &str, prop_ids: &mut HashSet<String>, problems: &mut Vec<String>) {
    let path = format!("rooms/{id}.lua");
    let room = match assets::text(&path) {
        None => return problems.push(format!("missing {path}")),
        Some(source) => match Room::from_lua(&source, &path) {
            Ok(room) => room,
            Err(error) => return problems.push(format!("{path}: {error}")),
        },
    };

    let script = format!("scripts/{id}.lua");
    match assets::text(&script) {
        None => problems.push(format!("missing {script}")),
        Some(source) => {
            let loaded = ScriptRunner::new().and_then(|mut runner| {
                runner.set_room(id)?;
                runner.load_definitions(&script, &source)
            });
            if let Err(error) = loaded {
                problems.push(format!("{script}: {error}"));
            }
        }
    }

    match room.background() {
        None => problems.push(format!("{id}: no background")),
        Some(image) if assets::bytes(image).is_none() => {
            problems.push(format!("{id}: missing {image}"));
        }
        Some(_) => {}
    }
    for prop in room.props() {
        if assets::bytes(&prop.image).is_none() {
            problems.push(format!("{id}: missing {}", prop.image));
        }
        if let Some(prop_id) = &prop.id
            && !prop_ids.insert(prop_id.clone())
        {
            problems.push(format!(
                "{id}: prop id {prop_id} is used twice (prop ids are global)"
            ));
        }
    }

    let mut hotspot_ids = HashSet::new();
    for hotspot in room.hotspots() {
        let name = format!("{id}.{}", hotspot.id);
        if !hotspot_ids.insert(hotspot.id.as_str()) {
            problems.push(format!("{name}: duplicate hotspot id"));
        }
        if hotspot.rect.w <= 0.0 || hotspot.rect.h <= 0.0 {
            problems.push(format!("{name}: empty rect"));
        }
        if let Some(spot) = hotspot.walk_to
            && room.constrain(spot.x, spot.y) != spot
        {
            problems.push(format!("{name}: walkTo is not walkable"));
        }
    }
}
