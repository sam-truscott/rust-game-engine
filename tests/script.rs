use game::actor::Actor;
use game::room::Room;
use game::script::ScriptRunner;
use macroquad::math::vec2;

const DELTA: f32 = 0.05;

fn floor() -> Room {
    Room::new(
        vec![
            vec2(0.0, 0.0),
            vec2(200.0, 0.0),
            vec2(200.0, 100.0),
            vec2(0.0, 100.0),
        ],
        vec![],
    )
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

#[test]
fn syntax_error_is_reported_when_starting() {
    let mut runner = ScriptRunner::new().unwrap();
    assert!(runner.start("broken", "walkTo(").is_err());
    assert!(!runner.is_running());
}
