use game::assets;
use game::room::Room;
use macroquad::math::vec2;

fn hall() -> Room {
    let source = assets::text("rooms/hall.lua").expect("rooms/hall.lua is missing");
    Room::from_lua(&source, "rooms/hall.lua").expect("rooms/hall.lua does not parse")
}

#[test]
fn hall_loads_with_its_background_obstacles_and_props() {
    let room = hall();
    assert_eq!(room.background(), Some("rooms/hall.png"));
    assert_eq!(room.outline().len(), 6);
    assert_eq!(room.obstacles().len(), 2);
    assert_eq!(room.props().len(), 8);
    assert_eq!(room.props()[0].depth, room.props()[0].y);
    assert_eq!(room.hotspots().len(), 10);
}

#[test]
fn a_prop_with_an_id_can_be_hidden_by_the_hotspot_with_the_same_id() {
    let room = hall();
    let door = room
        .props()
        .iter()
        .find(|prop| prop.id.as_deref() == Some("door"));
    assert!(door.is_some(), "the door prop has no id");
    assert!(room.hotspots().iter().any(|hotspot| hotspot.id == "door"));
}

#[test]
fn the_player_starts_somewhere_walkable() {
    let room = hall();
    assert_eq!(room.constrain(30.0, 50.0), vec2(30.0, 50.0));
}

#[test]
fn a_room_without_optional_fields_still_loads() {
    let room = Room::from_lua(
        "return { walkArea = { {0,0}, {10,0}, {10,10}, {0,10} } }",
        "tiny",
    )
    .unwrap();
    assert_eq!(room.background(), None);
    assert!(room.obstacles().is_empty());
    assert!(room.props().is_empty());
}

#[test]
fn a_room_missing_its_walk_area_is_an_error() {
    assert!(Room::from_lua("return {}", "empty").is_err());
}

fn closet() -> Room {
    let source = assets::text("rooms/closet.lua").expect("rooms/closet.lua is missing");
    Room::from_lua(&source, "rooms/closet.lua").expect("rooms/closet.lua does not parse")
}

#[test]
fn the_closet_is_a_small_dark_room_with_a_switch_door_and_shelf() {
    let room = closet();
    assert!(room.is_dark());
    assert!(!hall().is_dark());
    assert!(room.props().iter().any(|prop| prop.glow));
    let width = room.outline().iter().map(|p| p.x).fold(0.0, f32::max)
        - room.outline().iter().map(|p| p.x).fold(f32::MAX, f32::min);
    assert!(width <= 80.0, "the closet is a quarter of the screen wide");
    for id in ["switch", "door", "shelf"] {
        let hotspot = room.hotspots().iter().find(|h| h.id == id).unwrap();
        let spot = hotspot.walk_to.unwrap();
        assert_eq!(room.constrain(spot.x, spot.y), spot, "{id}");
    }
    assert!(assets::bytes(room.background().unwrap()).is_some());
    for prop in room.props() {
        assert!(assets::bytes(&prop.image).is_some(), "{}", prop.image);
    }
}

fn room_named(id: &str) -> Room {
    let path = format!("rooms/{id}.lua");
    let source = assets::text(&path).unwrap_or_else(|| panic!("{path} is missing"));
    Room::from_lua(&source, &path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[test]
fn every_room_has_its_script_images_and_reachable_hotspots() {
    for id in &game::manifest::Manifest::load().unwrap().rooms {
        let room = room_named(id);
        let path = format!("scripts/{id}.lua");
        let definitions = assets::text(&path).unwrap_or_else(|| panic!("{path} is missing"));
        let mut runner = game::script::ScriptRunner::new().unwrap();
        runner.set_room(id).unwrap();
        runner.load_definitions(&path, &definitions).unwrap();

        assert!(assets::bytes(room.background().unwrap()).is_some(), "{id}");
        for prop in room.props() {
            assert!(assets::bytes(&prop.image).is_some(), "{id}: {}", prop.image);
        }
        let mut ids: Vec<_> = room.hotspots().iter().map(|h| h.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(
            ids.len(),
            room.hotspots().len(),
            "{id}: duplicate hotspot id"
        );
        for hotspot in room.hotspots() {
            let spot = hotspot
                .walk_to
                .unwrap_or_else(|| panic!("{id}.{}: no spot", hotspot.id));
            assert_eq!(
                room.constrain(spot.x, spot.y),
                spot,
                "{id}.{} not walkable",
                hotspot.id
            );
            assert!(hotspot.rect.w > 0.0 && hotspot.rect.h > 0.0);
        }
    }
    assert!(assets::text("scripts/intro.lua").is_some());
    assert!(assets::bytes("characters/player.png").is_some());
}

#[test]
fn prop_ids_are_unique_across_rooms_because_the_world_keys_on_them() {
    let mut seen = std::collections::HashSet::new();
    for id in &game::manifest::Manifest::load().unwrap().rooms {
        for prop in room_named(id).props() {
            if let Some(prop_id) = &prop.id {
                assert!(seen.insert(prop_id.clone()), "prop id {prop_id} is reused");
            }
        }
    }
}

#[test]
fn an_outline_needs_three_finite_points() {
    for outline in ["{ {0,0}, {10,0} }", "{ {0,0}, {10,0}, {10,0/0} }", "{}"] {
        let source = format!("return {{ walkArea = {outline} }}");
        assert!(Room::from_lua(&source, "bad").is_err(), "{outline}");
    }
}

#[test]
fn asset_paths_cannot_leave_the_assets_folder() {
    assert!(assets::bytes("rooms/hall.lua").is_some());
    for path in [
        "../Cargo.toml",
        "rooms/../../Cargo.toml",
        "/rooms/hall.lua",
        r"rooms\hall.lua",
        "C:/Windows/win.ini",
    ] {
        assert!(assets::bytes(path).is_none(), "{path}");
    }
}

#[test]
fn rooms_and_items_are_loaded_without_file_or_os_access() {
    assert!(Room::from_lua("local f = io.open return {}", "x").is_err());
    assert!(Room::from_lua("return load('return 1')()", "x").is_err());
    assert!(game::items::Items::from_lua("os.exit()", "x").is_err());
}

#[test]
fn the_hall_window_shows_the_street_and_the_closet_has_none() {
    let street = hall()
        .street()
        .cloned()
        .expect("the hall has a street view");
    assert!(street.glass.w > 0.0 && street.mirror.is_some());
    assert!(street.floor < street.glass.y);
    assert!(closet().street().is_none());
}

#[test]
fn the_bundled_game_passes_the_check() {
    assert_eq!(game::check::check_game(), Vec::<String>::new());
}
