use game::actor::Actor;
use game::room::Room;
use game::script::ScriptRunner;
use macroquad::math::vec2;

const DELTA: f32 = 0.05;

fn floor_outline() -> Vec<macroquad::math::Vec2> {
    vec![
        vec2(0.0, 0.0),
        vec2(200.0, 0.0),
        vec2(200.0, 100.0),
        vec2(0.0, 100.0),
    ]
}

fn floor() -> Room {
    Room::new(floor_outline(), vec![])
}

fn step(runner: &mut ScriptRunner, room: &Room, actor: &mut Actor) {
    actor.update(DELTA);
    runner.update(DELTA, room, actor);
}

#[test]
fn script_walks_speaks_waits_and_finishes() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner
        .start("test", "walkTo(60, 50) say('hello') wait(0.5)")
        .unwrap();

    let mut spoke_at = None;
    for _ in 0..400 {
        step(&mut runner, &room, &mut actor);
        if spoke_at.is_none() && actor.is_speaking() {
            spoke_at = Some(actor.position());
        }
        if !runner.is_running() {
            break;
        }
    }

    assert!(!runner.is_running(), "script did not finish");
    let spoke_at = spoke_at.expect("never spoke");
    assert!(
        (spoke_at.x - 60.0).abs() < 1e-3,
        "spoke before arriving: {spoke_at}"
    );
}

#[test]
fn script_does_not_advance_while_the_action_runs() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner
        .start("test", "walkTo(190, 50) say('arrived')")
        .unwrap();

    for _ in 0..10 {
        step(&mut runner, &room, &mut actor);
    }
    assert!(actor.is_walking());
    assert!(!actor.is_speaking());
}

#[test]
fn clear_ends_scripts_that_are_suspended() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner.start("test", "wait(100)").unwrap();
    step(&mut runner, &room, &mut actor);
    assert!(runner.is_running());

    runner.clear();
    assert!(!runner.is_running());
}

#[test]
fn a_script_that_errors_is_dropped() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner.start("bad", "error('boom')").unwrap();

    step(&mut runner, &room, &mut actor);
    assert!(!runner.is_running());
}

#[test]
fn a_script_that_errors_does_not_stop_the_others() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner.start("bad", "error('boom')").unwrap();
    runner.start("good", "say('still here')").unwrap();

    step(&mut runner, &room, &mut actor);
    assert!(actor.is_speaking());
    assert!(runner.is_running());
}

/// Steps until the actor speaks; returns what it said and where it was standing.
fn run_until_speech(
    runner: &mut ScriptRunner,
    room: &Room,
    actor: &mut Actor,
) -> Option<(String, macroquad::math::Vec2)> {
    for _ in 0..600 {
        step(runner, room, actor);
        if let Some(text) = actor.speech() {
            return Some((text.to_owned(), actor.position()));
        }
    }
    None
}

#[test]
fn an_interaction_walks_to_the_spot_and_then_runs_the_handler() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner
        .load_definitions(
            "defs",
            "hotspot.box = { look = function() say('a box') end }",
        )
        .unwrap();
    runner
        .interact("box", "look", Some(vec2(60.0, 50.0)))
        .unwrap();

    let (said, at) = run_until_speech(&mut runner, &room, &mut actor).expect("never spoke");
    assert_eq!(said, "a box");
    assert!((at.x - 60.0).abs() < 1e-3, "spoke before arriving: {at}");
}

#[test]
fn a_handler_can_wait_and_speak_more_than_once() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner
        .load_definitions(
            "defs",
            "hotspot.box = { look = function() say('one') wait(0.2) say('two') end }",
        )
        .unwrap();
    runner.interact("box", "look", None).unwrap();

    let mut said = Vec::new();
    for _ in 0..400 {
        step(&mut runner, &room, &mut actor);
        if let Some(text) = actor.speech()
            && said.last().map(String::as_str) != Some(text)
        {
            said.push(text.to_owned());
        }
        if !runner.is_running() {
            break;
        }
    }
    assert_eq!(said, vec!["one", "two"]);
}

#[test]
fn without_a_handler_looking_and_other_verbs_get_default_replies() {
    let room = floor();
    for (verb, expected) in [
        ("look", "I see nothing special."),
        ("use", "I can't do that."),
        ("pickup", "I can't do that."),
    ] {
        let mut actor = Actor::new(10.0, 50.0);
        let mut runner = ScriptRunner::new().unwrap();
        runner
            .load_definitions(
                "defs",
                "hotspot.box = { open = function() say('opened') end }",
            )
            .unwrap();
        runner.interact("box", verb, None).unwrap();
        let (said, _) = run_until_speech(&mut runner, &room, &mut actor).expect("never spoke");
        assert_eq!(said, expected, "verb {verb}");
    }
}

#[test]
fn an_unknown_hotspot_gets_the_default_reply() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner.interact("nothing", "look", None).unwrap();
    let (said, _) = run_until_speech(&mut runner, &room, &mut actor).expect("never spoke");
    assert_eq!(said, "I see nothing special.");
}

#[test]
fn walking_to_a_hotspot_walks_without_speaking() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let mut runner = ScriptRunner::new().unwrap();
    runner
        .load_definitions(
            "defs",
            "hotspot.box = { walk = function() say('never') end }",
        )
        .unwrap();
    runner
        .interact("box", "walk", Some(vec2(60.0, 50.0)))
        .unwrap();

    for _ in 0..200 {
        step(&mut runner, &room, &mut actor);
        assert!(!actor.is_speaking());
        if !runner.is_running() {
            break;
        }
    }
    assert!(!runner.is_running());
    assert!((actor.position().x - 60.0).abs() < 1e-3);
}

#[test]
fn syntax_error_is_reported_when_starting() {
    let mut runner = ScriptRunner::new().unwrap();
    assert!(runner.start("broken", "walkTo(").is_err());
    assert!(!runner.is_running());
}

#[test]
fn animate_blocks_the_script_until_the_frames_have_played() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let world = game::world::World::default();
    let mut runner = ScriptRunner::with_world(world.clone()).unwrap();
    runner
        .start("test", "animate('door', 0, 3, 10) setFrame('lid', 2)")
        .unwrap();

    step(&mut runner, &room, &mut actor);
    assert!(world.is_animating("door"));
    assert_eq!(world.frame("lid"), 0);
    for _ in 0..30 {
        step(&mut runner, &room, &mut actor);
    }
    assert!(!runner.is_running());
    assert_eq!(world.frame("door"), 3);
    assert_eq!(world.frame("lid"), 2);
}

#[test]
fn animate_uses_the_props_fps_from_the_room_unless_given() {
    use game::room::Prop;
    let room = Room::with_options(
        floor_outline(),
        vec![],
        0.0,
        None,
        vec![Prop {
            id: Some("door".into()),
            image: "x.png".into(),
            x: 0.0,
            y: 0.0,
            depth: 0.0,
            frames: 4,
            fps: 2.0,
        }],
    );
    let mut actor = Actor::new(10.0, 50.0);
    let world = game::world::World::default();
    let mut runner = ScriptRunner::with_world(world.clone()).unwrap();
    runner.start("test", "animate('door', 0, 3)").unwrap();
    step(&mut runner, &room, &mut actor);
    // 4 frames at 2 fps take 2 seconds
    for _ in 0..30 {
        step(&mut runner, &room, &mut actor);
    }
    assert!(world.is_animating("door"));
    for _ in 0..20 {
        step(&mut runner, &room, &mut actor);
    }
    assert!(!world.is_animating("door"));
}
