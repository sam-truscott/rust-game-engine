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
    assert_eq!(room.outline().len(), 4);
    assert_eq!(room.obstacles().len(), 3);
    assert_eq!(room.props().len(), 4);
    assert_eq!(room.props()[0].depth, room.props()[0].y);
    assert_eq!(room.hotspots().len(), 4);
}

#[test]
fn a_prop_with_an_id_can_be_hidden_by_the_hotspot_with_the_same_id() {
    let room = hall();
    let key = room
        .props()
        .iter()
        .find(|prop| prop.id.as_deref() == Some("key"));
    assert!(key.is_some(), "the key prop has no id");
    assert!(room.hotspots().iter().any(|hotspot| hotspot.id == "key"));
}

#[test]
fn hotspots_are_reachable_and_the_room_script_loads() {
    let room = hall();
    let definitions = assets::text("scripts/hall.lua").expect("scripts/hall.lua is missing");
    let mut runner = game::script::ScriptRunner::new().unwrap();
    runner
        .load_definitions("scripts/hall.lua", &definitions)
        .unwrap();

    for hotspot in room.hotspots() {
        let spot = hotspot
            .walk_to
            .expect("hall hotspots have a spot to stand on");
        assert_eq!(
            room.constrain(spot.x, spot.y),
            spot,
            "{} is not walkable",
            hotspot.id
        );
        assert!(hotspot.rect.w > 0.0 && hotspot.rect.h > 0.0);
    }
}

#[test]
fn everything_the_hall_refers_to_exists() {
    let room = hall();
    assert!(assets::bytes(room.background().unwrap()).is_some());
    for prop in room.props() {
        assert!(
            assets::bytes(&prop.image).is_some(),
            "missing {}",
            prop.image
        );
    }
    assert!(assets::text("scripts/intro.lua").is_some());
    assert!(assets::bytes("characters/player.png").is_some());
}

#[test]
fn the_player_starts_somewhere_walkable() {
    let room = hall();
    assert_eq!(room.constrain(60.0, 50.0), vec2(60.0, 50.0));
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
