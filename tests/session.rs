use game::session::Session;
use game::ui::Verb;
use macroquad::math::{Vec2, vec2};

const STEP: f32 = 1.0 / 30.0;

const KEY: Vec2 = vec2(46.0, 39.0);
const COUNTER: Vec2 = vec2(160.0, 70.0);
const FLOOR: Vec2 = vec2(100.0, 85.0);
const LOOK_AT: Vec2 = vec2(80.0, 156.0);
const PICK_UP: Vec2 = vec2(132.0, 156.0);
const USE: Vec2 = vec2(184.0, 156.0);
const SLOT_0: Vec2 = vec2(16.0, 12.0);
const SLOT_1: Vec2 = vec2(44.0, 12.0);
const SLOT_5: Vec2 = vec2(156.0, 12.0);

fn settle(session: &mut Session) {
    for _ in 0..4000 {
        if !session.scripts.is_running() {
            return;
        }
        session.update(STEP);
    }
    panic!("scripts never finished");
}

/// Waits for a plain walk (which is not a script) to finish.
fn finish_walking(session: &mut Session) {
    for _ in 0..4000 {
        if !session.player.is_walking() {
            return;
        }
        session.update(STEP);
    }
    panic!("never stopped walking");
}

/// A session with the intro already over.
fn session() -> Session {
    let mut session = Session::load().unwrap();
    settle(&mut session);
    session
}

fn click(session: &mut Session, point: Vec2) {
    session.move_cursor(point);
    session.click(point);
}

fn next_words(session: &mut Session) -> String {
    for _ in 0..4000 {
        session.update(STEP);
        if let Some(text) = session.player.speech() {
            return text.to_owned();
        }
    }
    panic!("nobody spoke");
}

fn with_the_key() -> Session {
    let mut session = session();
    click(&mut session, PICK_UP);
    click(&mut session, KEY);
    settle(&mut session);
    session
}

#[test]
fn hovering_names_hotspots_and_the_sentence_follows() {
    let mut session = session();
    session.move_cursor(COUNTER);
    assert_eq!(session.hovered_name().as_deref(), Some("counter"));
    assert_eq!(session.sentence(), "Walk to counter");

    session.move_cursor(KEY);
    assert_eq!(session.sentence(), "Walk to brass key");

    session.move_cursor(FLOOR);
    assert_eq!(session.hovered_name(), None);
    assert_eq!(session.sentence(), "Walk to");

    session.move_cursor(LOOK_AT);
    assert_eq!(session.hovered_name(), None);
}

#[test]
fn walking_onto_an_object_walks_to_where_you_clicked_so_you_can_go_behind_it() {
    let mut session = session();
    let behind_the_window = vec2(256.0, 72.0);
    assert_eq!(
        session.hotspot_at(behind_the_window).map(|h| h.id.as_str()),
        Some("window")
    );

    click(&mut session, behind_the_window);
    assert!(session.player.is_walking());
    finish_walking(&mut session);
    assert_eq!(session.player.position(), behind_the_window);
}

#[test]
fn the_other_verbs_still_go_to_the_objects_standing_spot() {
    let mut session = session();
    click(&mut session, LOOK_AT);
    click(&mut session, vec2(256.0, 72.0));
    settle(&mut session);
    assert_eq!(session.player.position(), vec2(256.0, 40.0));
}

#[test]
fn actions_wait_while_the_intro_runs_but_choosing_a_verb_does_not() {
    let mut session = Session::load().unwrap();
    assert!(session.scripts.is_running());

    click(&mut session, FLOOR);
    assert!(!session.player.is_walking());

    click(&mut session, PICK_UP);
    assert_eq!(session.verb, Verb::PickUp);
}

#[test]
fn a_verb_can_be_chosen_while_an_action_is_still_playing_out() {
    let mut session = session();
    click(&mut session, PICK_UP);
    click(&mut session, KEY);
    assert!(session.scripts.is_running());
    assert_eq!(next_words(&mut session), "Got it.");
    assert!(session.scripts.is_running());

    click(&mut session, LOOK_AT);
    assert_eq!(session.verb, Verb::LookAt);

    settle(&mut session);
    assert_eq!(
        session.verb,
        Verb::LookAt,
        "the choice must survive the action ending"
    );
}

#[test]
fn a_verb_can_be_chosen_while_walking_to_something() {
    let mut session = session();
    click(&mut session, LOOK_AT);
    click(&mut session, COUNTER);
    session.update(STEP);
    assert!(session.scripts.is_running());
    assert!(session.player.is_walking());

    click(&mut session, USE);
    assert_eq!(session.verb, Verb::Use);
}

#[test]
fn an_item_can_be_picked_for_use_while_an_action_is_still_playing_out() {
    let mut session = session();
    click(&mut session, PICK_UP);
    click(&mut session, KEY);
    assert_eq!(next_words(&mut session), "Got it.");
    assert!(session.scripts.is_running());

    click(&mut session, USE);
    click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));

    settle(&mut session);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
    click(&mut session, COUNTER);
    assert_eq!(next_words(&mut session), "The key fits the drawer!");
}

#[test]
fn things_that_start_something_are_still_ignored_while_a_script_runs() {
    let mut session = session();
    click(&mut session, PICK_UP);
    click(&mut session, KEY);
    assert_eq!(next_words(&mut session), "Got it.");

    click(&mut session, LOOK_AT);
    click(&mut session, SLOT_0);
    click(&mut session, COUNTER);
    click(&mut session, FLOOR);
    assert!(!session.player.is_walking());
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert_eq!(session.player.position(), vec2(46.0, 30.0));
}

#[test]
fn picking_up_the_key_puts_it_in_the_inventory_and_removes_it_from_the_room() {
    let mut session = session();
    assert!(session.hotspot_at(KEY).is_some());

    click(&mut session, PICK_UP);
    assert_eq!(session.verb, Verb::PickUp);
    session.move_cursor(KEY);
    assert_eq!(session.sentence(), "Pick up brass key");
    click(&mut session, KEY);
    assert!(session.scripts.is_running());
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert!(session.world.is_hidden("key"));
    assert!(session.hotspot_at(KEY).is_none());
    assert_eq!(session.verb, Verb::WalkTo);
    assert_eq!(session.player.position(), vec2(46.0, 30.0));
}

#[test]
fn once_picked_up_clicking_where_the_key_was_just_walks() {
    let mut session = with_the_key();
    click(&mut session, KEY);
    assert!(session.player.is_walking());
    assert!(!session.scripts.is_running());
}

#[test]
fn the_carried_item_has_a_name_and_a_slot() {
    let mut session = with_the_key();
    session.move_cursor(SLOT_0);
    assert_eq!(session.hovered_name().as_deref(), Some("brass key"));
    session.move_cursor(SLOT_1);
    assert_eq!(session.hovered_name(), None);
}

#[test]
fn a_plain_click_on_a_carried_item_looks_at_it() {
    let mut session = with_the_key();
    session.move_cursor(SLOT_0);
    assert_eq!(session.verb, Verb::WalkTo);
    assert_eq!(session.sentence(), "Look at brass key");

    click(&mut session, SLOT_0);
    assert_eq!(next_words(&mut session), "It must open something.");
}

#[test]
fn looking_at_a_carried_item_runs_its_handler() {
    let mut session = with_the_key();
    click(&mut session, LOOK_AT);
    click(&mut session, SLOT_0);
    assert_eq!(next_words(&mut session), "It must open something.");
    settle(&mut session);
    assert_eq!(session.verb, Verb::WalkTo);
}

#[test]
fn the_key_opens_the_counter_drawer_and_is_swapped_for_a_coin() {
    let mut session = with_the_key();

    click(&mut session, USE);
    click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
    assert!(!session.scripts.is_running());
    assert_eq!(session.sentence(), "Use brass key");

    session.move_cursor(FLOOR);
    assert_eq!(session.sentence(), "Use brass key with");

    session.move_cursor(COUNTER);
    assert_eq!(session.sentence(), "Use brass key with counter");

    click(&mut session, COUNTER);
    assert_eq!(next_words(&mut session), "The key fits the drawer!");
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["coin"]);
    assert_eq!(session.holding, None);
    assert_eq!(session.verb, Verb::WalkTo);
    assert_eq!(session.player.position(), vec2(160.0, 30.0));
}

#[test]
fn clicking_the_room_where_nothing_is_cancels_a_held_item_without_walking() {
    let mut session = with_the_key();
    click(&mut session, USE);
    click(&mut session, SLOT_0);

    click(&mut session, FLOOR);
    assert_eq!(session.holding, None);
    assert_eq!(session.verb, Verb::WalkTo);
    assert!(!session.player.is_walking());
    assert_eq!(session.world.items(), vec!["brass_key"]);
}

#[test]
fn picking_a_verb_puts_a_held_item_back() {
    let mut session = with_the_key();
    click(&mut session, USE);
    click(&mut session, SLOT_0);
    assert!(session.holding.is_some());

    click(&mut session, LOOK_AT);
    assert_eq!(session.holding, None);
    assert_eq!(session.verb, Verb::LookAt);
}

#[test]
fn clicking_the_held_item_again_uses_it_on_its_own() {
    let mut session = with_the_key();
    click(&mut session, USE);
    click(&mut session, SLOT_0);
    click(&mut session, SLOT_0);

    assert_eq!(session.holding, None);
    assert_eq!(next_words(&mut session), "I can't do that.");
}

#[test]
fn using_one_item_on_another_with_no_handler_does_not_work() {
    let mut session = with_the_key();
    session.world.give("coin");

    click(&mut session, USE);
    click(&mut session, SLOT_0);
    click(&mut session, SLOT_1);
    assert_eq!(next_words(&mut session), "That doesn't work.");
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key", "coin"]);
}

#[test]
fn an_empty_slot_does_nothing() {
    let mut session = with_the_key();
    click(&mut session, LOOK_AT);
    click(&mut session, SLOT_5);
    assert!(!session.scripts.is_running());
    assert_eq!(session.verb, Verb::LookAt);
}

#[test]
fn using_the_wrong_item_on_a_hotspot_does_not_work() {
    let mut session = with_the_key();
    session.world.give("coin");
    click(&mut session, USE);
    click(&mut session, SLOT_1);
    click(&mut session, COUNTER);
    assert_eq!(next_words(&mut session), "That doesn't work.");
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key", "coin"]);
}
