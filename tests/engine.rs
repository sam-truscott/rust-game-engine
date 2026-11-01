//! The engine against a tiny game of its own (tests/fixtures/mini), so none of this depends on the
//! pub: the manifest, the defaults it sets, rooms found by id, and the check.

use macroquad::math::vec2;
use wumpus_g_engine::manifest::Manifest;
use wumpus_g_engine::session::Session;
use wumpus_g_engine::ui;

const STEP: f32 = 1.0 / 30.0;

fn mini() -> Session {
    wumpus_g_engine::assets::use_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/mini"));
    Session::load().unwrap()
}

fn settle(session: &mut Session) {
    for _ in 0..4000 {
        if !session.scripts.is_running() {
            return;
        }
        session.update(STEP);
    }
    panic!("scripts never finished");
}

fn double_click(session: &mut Session, room_point: macroquad::math::Vec2) {
    let at = ui::to_screen(room_point);
    session.click(at);
    session.click(at);
}

#[test]
fn the_manifest_decides_the_start_and_the_rooms() {
    let session = mini();
    assert_eq!(session.manifest.title, "Mini");
    assert_eq!(session.room_id, "a");
    assert_eq!(session.manifest.rooms, ["a", "b"]);
    assert_eq!(session.player.position(), vec2(20.0, 20.0));
}

#[test]
fn the_manifest_tuning_reaches_the_player() {
    let mut session = mini();
    session.scripts.clear();
    session.player.walk_to(&session.room, 100.0, 20.0);
    session.update(0.5);
    // 120 units a second for half a second from x = 20 (the default 60 would reach 50).
    assert!((session.player.position().x - 80.0).abs() < 0.01);
}

#[test]
fn the_intro_runs_from_the_manifest() {
    let mut session = mini();
    session.update(0.1);
    assert_eq!(session.player.speech(), Some("Hello from Mini."));
    settle(&mut session);
    assert!(session.player.speech().is_none());
}

#[test]
fn a_door_changes_room_and_back_without_the_engine_knowing_the_names() {
    let mut session = mini();
    settle(&mut session);
    double_click(&mut session, vec2(160.0, 30.0));
    settle(&mut session);
    assert_eq!(session.room_id, "b");
    double_click(&mut session, vec2(10.0, 20.0));
    settle(&mut session);
    assert_eq!(session.room_id, "a");
}

#[test]
fn a_look_without_a_handler_says_the_manifest_line() {
    let mut session = mini();
    settle(&mut session);
    // The back hotspot of room b has no look handler.
    double_click(&mut session, vec2(160.0, 30.0));
    settle(&mut session);
    session.click(ui::to_screen(vec2(10.0, 20.0)));
    session.update(STEP);
    for _ in 0..200 {
        if session.player.speech().is_some() {
            break;
        }
        session.update(STEP);
    }
    assert_eq!(session.player.speech(), Some("Nothing."));
}

#[test]
fn a_save_in_a_room_the_game_does_not_have_is_refused() {
    let session = mini();
    let mut data = session.capture();
    data.room = "pub_hall".to_owned();
    assert!(Session::from_save(&data).is_err());
}

#[test]
fn the_fixture_game_passes_the_check() {
    mini();
    assert_eq!(wumpus_g_engine::check::check_game(), Vec::<String>::new());
}

#[test]
fn the_check_reports_a_broken_game() {
    wumpus_g_engine::assets::use_dir(env!("CARGO_MANIFEST_DIR"));
    assert!(!wumpus_g_engine::check::check_game().is_empty());
    wumpus_g_engine::assets::use_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"));
    assert!(!wumpus_g_engine::check::check_game().is_empty());
}

#[test]
fn a_manifest_must_name_a_start_room_it_lists() {
    let source = r#"return { id = "x", player = "p", cursor = "c",
        rooms = { "a" }, start = { room = "z", x = 1, y = 1 } }"#;
    assert!(Manifest::from_lua(source, "x").is_err());
}

#[test]
fn manifest_numbers_must_be_positive() {
    let source = r#"return { id = "x", player = "p", cursor = "c", rooms = { "a" },
        start = { x = 1, y = 1 }, tuning = { walk_speed = 0 } }"#;
    assert!(Manifest::from_lua(source, "x").is_err());
}

#[test]
fn a_street_can_be_dressed_from_the_room_file() {
    let source = r#"return {
        walkArea = { {0,0}, {10,0}, {10,10} },
        street = { glass = {0, 5, 20, 8}, gap = {1, 2}, crossing = {1, 1.5}, first = {0.5, 0.5},
          car = { pattern = {"BB", "WW"}, bodies = {{1, 0, 0}}, block = 2 },
          light = { wash_strength = 0.5 } } }"#;
    let street = wumpus_g_engine::room::Room::from_lua(source, "x")
        .unwrap()
        .street()
        .cloned()
        .unwrap();
    assert_eq!(street.timing.gap, (1.0, 2.0));
    assert_eq!(street.look.pattern, ["BB", "WW"]);
    assert_eq!(street.look.block, 2.0);
    assert_eq!(street.look.wash_strength, 0.5);

    for bad in [
        "gap = {2, 1}",
        "car = { pattern = {\"BB\", \"W\"} }",
        "car = { bodies = {} }",
        "car = { block = 0 }",
    ] {
        let source = format!(
            "return {{ walkArea = {{ {{0,0}}, {{10,0}}, {{10,10}} }}, street = {{ glass = {{0,0,1,1}}, {bad} }} }}"
        );
        assert!(
            wumpus_g_engine::room::Room::from_lua(&source, "x").is_err(),
            "{bad}"
        );
    }
}

#[test]
fn a_script_starts_a_named_screen_effect() {
    let mut session = mini();
    session.scripts.clear();
    session
        .scripts
        .start("t", r#"screenEffect("glitch", 2) wait(5)"#)
        .unwrap();
    session.update(STEP);
    session.update(1.0);
    let effect = session.world.screen_effect().expect("effect is running");
    assert_eq!(effect.name, "glitch");
    assert!((effect.progress - 0.5).abs() < 0.05);
    session.scripts.clear();
    for bad in [r#"screenEffect("../x", 1)"#, r#"screenEffect("x", 0)"#] {
        assert!(session.scripts.start("t", bad).is_ok());
        session.update(STEP);
        session.scripts.clear();
    }
}
