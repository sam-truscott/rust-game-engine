use game::session::Session;
use game::ui::{self, PANEL_HEIGHT};
use macroquad::math::{Vec2, vec2};

const STEP: f32 = 1.0 / 30.0;

// Clicks are screen points: the room is drawn above the panel, so a room point has the panel's height
// added to its y. The inventory slots are inside the panel.
const KEY: Vec2 = vec2(78.0, 81.0 + PANEL_HEIGHT);
const BAR: Vec2 = vec2(240.0, 70.0 + PANEL_HEIGHT);
const CUPBOARD: Vec2 = vec2(44.0, 100.0 + PANEL_HEIGHT);
const FLOOR: Vec2 = vec2(110.0, 85.0 + PANEL_HEIGHT);
const WINDOW: Vec2 = vec2(96.0, 120.0 + PANEL_HEIGHT);
const DOOR: Vec2 = vec2(146.0, 110.0 + PANEL_HEIGHT);
const SLOT_0: Vec2 = vec2(16.0, 13.0);
const SLOT_1: Vec2 = vec2(44.0, 13.0);
const SLOT_5: Vec2 = vec2(156.0, 13.0);

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

/// A single click: the time moves on first, so it never counts as the second half of a double click.
fn click(session: &mut Session, point: Vec2) {
    session.update(1.0);
    session.move_cursor(point);
    session.click(point);
}

/// Two clicks in quick succession.
fn double_click(session: &mut Session, point: Vec2) {
    click(session, point);
    session.update(STEP);
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
    double_click(&mut session, KEY);
    settle(&mut session);
    session
}

#[test]
fn hotspots_are_found_by_their_room_position() {
    let session = session();
    let at = |point: Vec2| {
        session
            .hotspot_at(ui::to_room(point))
            .map(|hotspot| hotspot.id.as_str())
    };
    assert_eq!(at(BAR), Some("bar"));
    assert_eq!(at(KEY), Some("key"));
    assert_eq!(at(WINDOW), Some("window"));
    assert_eq!(at(FLOOR), None);
}

#[test]
fn a_click_on_the_panel_never_walks_but_one_just_above_it_does() {
    let mut session = session();
    click(&mut session, vec2(150.0, 20.0));
    click(&mut session, vec2(150.0, PANEL_HEIGHT - 0.5));
    assert!(!session.player.is_walking());

    click(&mut session, FLOOR);
    assert!(session.player.is_walking());
}

#[test]
fn clicking_where_nothing_is_walks_there() {
    let mut session = session();
    click(&mut session, FLOOR);
    assert!(session.player.is_walking());
    finish_walking(&mut session);
    assert_eq!(session.player.position(), ui::to_room(FLOOR));
}

#[test]
fn a_single_click_on_an_object_looks_at_it_from_its_standing_spot() {
    let mut session = session();
    click(&mut session, BAR);
    assert_eq!(
        next_words(&mut session),
        "A long oak bar, ring-stained and well loved."
    );
    assert_eq!(session.player.position(), vec2(240.0, 40.0));
}

#[test]
fn a_double_click_on_an_object_interacts_instead_of_looking() {
    let mut session = session();
    double_click(&mut session, BAR);
    assert_eq!(next_words(&mut session), "Nobody is serving.");
    settle(&mut session);
    assert_eq!(session.player.position(), vec2(240.0, 40.0));
}

#[test]
fn a_double_click_cuts_short_the_look_the_first_click_started() {
    let mut session = session();
    click(&mut session, BAR);
    session.update(STEP * 3.0);
    assert!(session.scripts.is_running());
    session.click(BAR);
    assert_eq!(next_words(&mut session), "Nobody is serving.");
}

#[test]
fn two_slow_clicks_are_not_a_double_click() {
    let mut session = session();
    click(&mut session, BAR);
    session.update(1.0);
    session.click(BAR);
    assert_eq!(
        next_words(&mut session),
        "A long oak bar, ring-stained and well loved.",
        "the second click is ignored while the look plays"
    );
}

#[test]
fn a_third_quick_click_starts_a_new_double_click() {
    let mut session = session();
    double_click(&mut session, BAR);
    settle(&mut session);
    session.click(BAR);
    assert_eq!(
        next_words(&mut session),
        "A long oak bar, ring-stained and well loved."
    );
}

#[test]
fn clicks_wait_while_the_intro_runs() {
    let mut session = Session::load().unwrap();
    assert!(session.scripts.is_running());

    click(&mut session, FLOOR);
    assert!(!session.player.is_walking());
}

#[test]
fn single_clicks_are_ignored_while_a_script_runs() {
    let mut session = session();
    double_click(&mut session, KEY);
    assert_eq!(next_words(&mut session), "Got it.");

    session.click(SLOT_0);
    session.click(BAR);
    session.click(FLOOR);
    assert!(!session.player.is_walking());
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert_eq!(session.player.position(), vec2(78.0, 70.0));
}

#[test]
fn an_item_can_be_picked_while_an_action_is_still_playing_out() {
    let mut session = session();
    double_click(&mut session, KEY);
    assert_eq!(next_words(&mut session), "Got it.");
    assert!(session.scripts.is_running());

    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
    assert!(!session.scripts.is_running());

    click(&mut session, CUPBOARD);
    assert_eq!(next_words(&mut session), "The key fits the cupboard!");
}

#[test]
fn interacting_with_the_key_puts_it_in_the_inventory_and_removes_it_from_the_room() {
    let mut session = session();
    assert!(session.hotspot_at(ui::to_room(KEY)).is_some());

    double_click(&mut session, KEY);
    assert!(session.scripts.is_running());
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert!(session.world.is_hidden("key"));
    assert!(session.hotspot_at(ui::to_room(KEY)).is_none());
    assert_eq!(session.player.position(), vec2(78.0, 70.0));
}

#[test]
fn once_picked_up_clicking_where_the_key_was_just_walks() {
    let mut session = with_the_key();
    click(&mut session, KEY);
    assert!(session.player.is_walking());
    assert!(!session.scripts.is_running());
}

#[test]
fn a_single_click_on_a_carried_item_looks_at_it() {
    let mut session = with_the_key();
    click(&mut session, SLOT_0);
    assert_eq!(next_words(&mut session), "It must open something.");
    settle(&mut session);
    assert_eq!(session.holding, None);
}

#[test]
fn a_double_click_on_a_carried_item_picks_it_up_to_use() {
    let mut session = with_the_key();
    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
    assert_eq!(session.player.speech(), None);
    assert!(!session.scripts.is_running());
}

#[test]
fn the_key_opens_the_cupboard_and_is_swapped_for_a_coin() {
    let mut session = with_the_key();

    double_click(&mut session, SLOT_0);
    click(&mut session, CUPBOARD);
    assert_eq!(next_words(&mut session), "The key fits the cupboard!");
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["coin"]);
    assert_eq!(session.holding, None);
    assert_eq!(session.player.position(), vec2(44.0, 70.0));
}

#[test]
fn clicking_the_room_where_nothing_is_cancels_a_held_item_without_walking() {
    let mut session = with_the_key();
    double_click(&mut session, SLOT_0);

    click(&mut session, FLOOR);
    assert_eq!(session.holding, None);
    assert!(!session.player.is_walking());
    assert_eq!(session.world.items(), vec!["brass_key"]);
}

#[test]
fn clicking_the_held_item_again_uses_it_on_its_own() {
    let mut session = with_the_key();
    double_click(&mut session, SLOT_0);
    click(&mut session, SLOT_0);

    assert_eq!(session.holding, None);
    assert_eq!(next_words(&mut session), "I can't do that.");
}

#[test]
fn using_one_item_on_another_with_no_handler_does_not_work() {
    let mut session = with_the_key();
    session.world.give("coin");

    double_click(&mut session, SLOT_0);
    click(&mut session, SLOT_1);
    assert_eq!(next_words(&mut session), "That doesn't work.");
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key", "coin"]);
}

#[test]
fn an_empty_slot_does_nothing() {
    let mut session = with_the_key();
    double_click(&mut session, SLOT_5);
    assert!(!session.scripts.is_running());
    assert_eq!(session.holding, None);
}

#[test]
fn using_the_wrong_item_on_a_hotspot_does_not_work() {
    let mut session = with_the_key();
    session.world.give("coin");
    double_click(&mut session, SLOT_1);
    click(&mut session, CUPBOARD);
    assert_eq!(next_words(&mut session), "That doesn't work.");
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key", "coin"]);
}

// The closet: its door and switch, as screen points.
const CLOSET_SWITCH: Vec2 = vec2(163.0, 60.0 + PANEL_HEIGHT);
const CLOSET_DOOR: Vec2 = vec2(182.0, 50.0 + PANEL_HEIGHT);
const CLOSET_SHELF: Vec2 = vec2(140.0, 80.0 + PANEL_HEIGHT);

fn in_the_closet() -> Session {
    let mut session = session();
    double_click(&mut session, DOOR);
    settle(&mut session);
    session
}

#[test]
fn a_double_click_on_the_door_opens_it_and_goes_into_the_closet() {
    let mut session = session();
    assert_eq!(session.room_id, "hall");
    double_click(&mut session, DOOR);
    settle(&mut session);
    for _ in 0..3 {
        session.update(STEP);
    }

    assert_eq!(session.room_id, "closet");
    assert!(session.take_room_changed());
    assert!(!session.take_room_changed(), "the change is reported once");
    assert_eq!(session.player.position(), vec2(182.0, 28.0));
}

#[test]
fn the_closet_is_dark_until_the_switch_is_used() {
    let mut session = in_the_closet();
    assert!(session.is_dark());
    assert_eq!(session.world.frame("closet_switch"), 0);

    click(&mut session, CLOSET_SHELF);
    assert_eq!(next_words(&mut session), "I can't see a thing in here.");
    settle(&mut session);

    double_click(&mut session, CLOSET_SWITCH);
    settle(&mut session);
    assert!(!session.is_dark());
    assert_eq!(session.world.frame("closet_switch"), 1);

    click(&mut session, CLOSET_SHELF);
    assert_eq!(
        next_words(&mut session),
        "Shelves of clean glasses for the pub."
    );
}

#[test]
fn the_switch_turns_the_light_off_again() {
    let mut session = in_the_closet();
    for expected_dark in [false, true] {
        double_click(&mut session, CLOSET_SWITCH);
        settle(&mut session);
        assert_eq!(session.is_dark(), expected_dark);
    }
}

#[test]
fn the_lights_stay_on_when_you_leave_and_come_back() {
    let mut session = in_the_closet();
    double_click(&mut session, CLOSET_SWITCH);
    settle(&mut session);

    double_click(&mut session, CLOSET_DOOR);
    settle(&mut session);
    session.update(STEP);
    assert_eq!(session.room_id, "hall");
    assert!(!session.is_dark(), "the hall is not a dark room");

    double_click(&mut session, DOOR);
    settle(&mut session);
    session.update(STEP);
    assert_eq!(session.room_id, "closet");
    assert!(!session.is_dark());
}

#[test]
fn the_closet_door_leads_back_to_the_bar_with_the_door_closed() {
    let mut session = in_the_closet();
    double_click(&mut session, CLOSET_DOOR);
    settle(&mut session);
    session.update(STEP);

    assert_eq!(session.room_id, "hall");
    assert_eq!(session.player.position(), vec2(146.0, 90.0));
    assert_eq!(session.world.frame("door"), 0);
    assert_eq!(session.world.frame("closet_door"), 0);
}

#[test]
fn each_room_keeps_its_own_hotspot_handlers() {
    let mut session = in_the_closet();
    click(&mut session, CLOSET_DOOR);
    assert_eq!(next_words(&mut session), "The door back to the bar.");
    settle(&mut session);
    double_click(&mut session, CLOSET_DOOR);
    settle(&mut session);
    session.update(STEP);

    click(&mut session, DOOR);
    assert_eq!(next_words(&mut session), "The door to the cupboard.");
}
