use game::actor::Actor;
use game::room::Room;
use game::script::ScriptRunner;
use game::world::World;
use macroquad::math::vec2;

const DELTA: f32 = 0.05;

mod common;

use common::floor;

/// Runs until nothing is left running; returns everything the actor said, in order.
fn run_to_the_end(runner: &mut ScriptRunner, room: &Room, actor: &mut Actor) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    for _ in 0..2000 {
        actor.update(DELTA);
        runner.update(DELTA, room, actor);
        if let Some(text) = actor.speech()
            && said.last().map(String::as_str) != Some(text)
        {
            said.push(text.to_owned());
        }
        if !runner.is_running() {
            return said;
        }
    }
    panic!("scripts never finished; said {said:?}");
}

fn runner_with_world() -> (ScriptRunner, World) {
    let world = World::default();
    (ScriptRunner::with_world(world.clone()).unwrap(), world)
}

#[test]
fn scripts_can_give_take_and_ask_about_items() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, world) = runner_with_world();
    runner
        .start(
            "test",
            "give('key')
             give('coin')
             if has('key') then say('have key') end
             if take('coin') then say('took coin') end
             if not has('coin') then say('no coin') end
             if not take('coin') then say('nothing to take') end",
        )
        .unwrap();

    let said = run_to_the_end(&mut runner, &room, &mut actor);
    assert_eq!(
        said,
        vec!["have key", "took coin", "no coin", "nothing to take"]
    );
    assert_eq!(world.items(), vec!["key"]);
}

#[test]
fn scripts_can_hide_things() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, world) = runner_with_world();
    runner.start("test", "hide('key')").unwrap();
    run_to_the_end(&mut runner, &room, &mut actor);
    assert!(world.is_hidden("key"));
}

#[test]
fn the_runner_gives_out_its_world() {
    let (runner, world) = runner_with_world();
    world.give("key");
    assert!(runner.world().has("key"));
}

#[test]
fn an_item_handler_runs_without_walking() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, _) = runner_with_world();
    runner
        .load_definitions("defs", "item.key = { look = function() say('shiny') end }")
        .unwrap();
    runner.interact_item("key", "look").unwrap();

    let said = run_to_the_end(&mut runner, &room, &mut actor);
    assert_eq!(said, vec!["shiny"]);
    assert_eq!(actor.position(), vec2(10.0, 50.0));
}

#[test]
fn items_without_a_handler_get_the_default_replies() {
    let room = floor();
    for (verb, expected) in [
        ("look", "I see nothing special."),
        ("interact", "I can't do that."),
    ] {
        let mut actor = Actor::new(10.0, 50.0);
        let (mut runner, _) = runner_with_world();
        runner.interact_item("key", verb).unwrap();
        let said = run_to_the_end(&mut runner, &room, &mut actor);
        assert_eq!(said, vec![expected], "verb {verb}");
    }
}

#[test]
fn using_an_item_on_a_hotspot_walks_there_and_runs_its_use_with_handler() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, world) = runner_with_world();
    runner
        .load_definitions(
            "defs",
            "hotspot.door = { use_with = { key = function()
               say('unlocked')
               take('key')
               give('coin')
             end } }",
        )
        .unwrap();
    world.give("key");
    runner
        .use_with_hotspot("key", "door", Some(vec2(60.0, 50.0)))
        .unwrap();

    let mut spoke_at = None;
    for _ in 0..600 {
        actor.update(DELTA);
        runner.update(DELTA, &room, &mut actor);
        if spoke_at.is_none() && actor.is_speaking() {
            spoke_at = Some(actor.position());
        }
        if !runner.is_running() {
            break;
        }
    }
    let spoke_at = spoke_at.expect("never spoke");
    assert!(
        (spoke_at.x - 60.0).abs() < 1e-3,
        "spoke before arriving: {spoke_at}"
    );
    assert_eq!(world.items(), vec!["coin"]);
}

#[test]
fn using_an_item_on_another_item_runs_that_items_use_with_handler() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, world) = runner_with_world();
    runner
        .load_definitions(
            "defs",
            "item.rope = { use_with = { hook = function() say('tied'); give('line') end } }",
        )
        .unwrap();
    world.give("hook");
    world.give("rope");
    runner.use_with_item("hook", "rope").unwrap();

    let said = run_to_the_end(&mut runner, &room, &mut actor);
    assert_eq!(said, vec!["tied"]);
    assert!(world.has("line"));
}

#[test]
fn a_combination_nobody_handles_gets_the_default_reply() {
    let room = floor();
    let mut actor = Actor::new(10.0, 50.0);
    let (mut runner, _) = runner_with_world();
    runner.use_with_item("hook", "rope").unwrap();
    let said = run_to_the_end(&mut runner, &room, &mut actor);
    assert_eq!(said, vec!["That doesn't work."]);

    let mut runner = ScriptRunner::new().unwrap();
    runner
        .use_with_hotspot("key", "nothing", Some(vec2(20.0, 50.0)))
        .unwrap();
    let said = run_to_the_end(&mut runner, &room, &mut actor);
    assert_eq!(said, vec!["That doesn't work."]);
}
