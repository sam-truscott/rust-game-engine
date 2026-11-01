use macroquad::math::{Vec2, vec2};
use wumpus_g_engine::room::{Hotspot, Room};
use wumpus_g_engine::session::Session;
use wumpus_g_engine::ui::{self, PANEL_HEIGHT};

const STEP: f32 = 1.0 / 30.0;

/// A room's hotspot, read from the same room file the game uses, so the tests follow the layout.
fn hotspot(room: &str, id: &str) -> Hotspot {
    let path = format!("rooms/{room}.lua");
    let source = wumpus_g_engine::assets::text(&path).unwrap();
    let room = Room::from_lua(&source, &path).unwrap();
    room.hotspots().iter().find(|h| h.id == id).unwrap().clone()
}

/// A screen point in the middle of a hotspot, to click it.
fn point(room: &str, id: &str) -> Vec2 {
    ui::to_screen(hotspot(room, id).rect.center())
}

/// Where the player stands to use a hotspot.
fn stand(room: &str, id: &str) -> Vec2 {
    hotspot(room, id).walk_to.unwrap()
}

// Clicks are screen points: the room is drawn above the panel, so a room point has the panel's height
// added to its y. The inventory slots are inside the panel.
const FLOOR: Vec2 = vec2(110.0, 85.0 + PANEL_HEIGHT);
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

/// A session in the closet with the light on, where the key is.
fn in_the_lit_closet() -> Session {
    let mut session = session();
    double_click(&mut session, point("hall", "door"));
    settle(&mut session);
    assert_eq!(session.room_id, "closet");
    session.world.set_light("closet", true);
    session
}

/// A session back in the hall carrying the key from the closet shelf.
fn with_the_key() -> Session {
    let mut session = in_the_lit_closet();
    double_click(&mut session, point("closet", "key"));
    settle(&mut session);
    double_click(&mut session, point("closet", "door"));
    settle(&mut session);
    assert_eq!(session.room_id, "hall");
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
    assert_eq!(at(point("hall", "bar")), Some("bar"));
    assert_eq!(at(point("hall", "door")), Some("door"));
    assert_eq!(at(point("hall", "window")), Some("window"));
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
    click(&mut session, point("hall", "bar"));
    assert_eq!(
        next_words(&mut session),
        "A long oak bar, ring-stained and well loved."
    );
    assert_eq!(session.player.position(), stand("hall", "bar"));
}

#[test]
fn a_double_click_on_an_object_interacts_instead_of_looking() {
    let mut session = session();
    double_click(&mut session, point("hall", "bar"));
    assert_eq!(next_words(&mut session), "Nobody is serving.");
    settle(&mut session);
    assert_eq!(session.player.position(), stand("hall", "bar"));
}

#[test]
fn a_double_click_cuts_short_the_look_the_first_click_started() {
    let mut session = session();
    click(&mut session, point("hall", "bar"));
    session.update(STEP * 3.0);
    assert!(session.scripts.is_running());
    session.click(point("hall", "bar"));
    assert_eq!(next_words(&mut session), "Nobody is serving.");
}

#[test]
fn two_slow_clicks_are_not_a_double_click() {
    let mut session = session();
    click(&mut session, point("hall", "bar"));
    session.update(1.0);
    session.click(point("hall", "bar"));
    assert_eq!(
        next_words(&mut session),
        "A long oak bar, ring-stained and well loved.",
        "the second click is ignored while the look plays"
    );
}

#[test]
fn a_click_skips_a_line_of_a_look_and_starts_nothing() {
    let mut session = session();
    click(&mut session, point("hall", "window"));
    assert_eq!(
        next_words(&mut session),
        "A road, and a hedge on the other side."
    );
    session.click(FLOOR);
    assert_eq!(session.player.speech(), None);
    assert!(!session.player.is_walking());
    assert_eq!(next_words(&mut session), "Nobody about.");
    session.click(point("hall", "window"));
    session.update(STEP);
    assert!(!session.scripts.is_running());
    assert_eq!(session.player.position(), stand("hall", "window"));
}

#[test]
fn a_click_skips_the_intro() {
    let mut session = Session::load().unwrap();
    session.update(0.5);
    assert!(session.player.speech().is_some());
    session.click(FLOOR);
    assert_eq!(session.player.speech(), None);
    session.update(STEP);
    assert!(!session.scripts.is_running());
    assert!(!session.player.is_walking());
}

#[test]
fn a_click_just_after_a_look_starts_does_not_skip_it() {
    let mut session = with_the_key();
    click(&mut session, SLOT_0);
    assert_eq!(next_words(&mut session), "It must open something.");
    session.click(FLOOR);
    assert_eq!(
        session.player.speech(),
        Some("It must open something."),
        "within a double click's time of the look starting"
    );
    session.update(0.5);
    session.click(FLOOR);
    assert_eq!(session.player.speech(), None);
}

#[test]
fn a_third_quick_click_starts_a_new_double_click() {
    let mut session = session();
    double_click(&mut session, point("hall", "bar"));
    settle(&mut session);
    session.click(point("hall", "bar"));
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
    let mut session = in_the_lit_closet();
    double_click(&mut session, point("closet", "key"));
    assert_eq!(next_words(&mut session), "Got it.");

    session.click(SLOT_0);
    session.click(point("closet", "shelf"));
    session.click(FLOOR);
    assert!(!session.player.is_walking());
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert_eq!(session.player.position(), stand("closet", "key"));
}

#[test]
fn clicks_are_ignored_while_an_action_is_playing_out() {
    let mut session = in_the_lit_closet();
    double_click(&mut session, point("closet", "key"));
    assert_eq!(next_words(&mut session), "Got it.");
    assert!(session.scripts.is_running());
    assert!(session.input_locked());

    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding, None);

    settle(&mut session);
    assert!(!session.input_locked());
    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
}

#[test]
fn interacting_with_the_key_puts_it_in_the_inventory_and_removes_it_from_the_room() {
    let mut session = in_the_lit_closet();
    assert!(
        session
            .hotspot_at(ui::to_room(point("closet", "key")))
            .is_some()
    );

    double_click(&mut session, point("closet", "key"));
    assert!(session.scripts.is_running());
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["brass_key"]);
    assert!(session.world.is_hidden("key"));
    let under = session
        .hotspot_at(ui::to_room(point("closet", "key")))
        .map(|hotspot| hotspot.id.as_str());
    assert_eq!(under, Some("shelf"));
    assert_eq!(session.player.position(), stand("closet", "key"));
}

#[test]
fn the_key_cannot_be_taken_in_the_dark() {
    let mut session = in_the_lit_closet();
    session.world.set_light("closet", false);
    double_click(&mut session, point("closet", "key"));
    assert_eq!(next_words(&mut session), "I can't see a thing in here.");
    settle(&mut session);
    assert!(session.world.items().is_empty());
    assert!(!session.world.is_hidden("key"));
}

#[test]
fn once_picked_up_clicking_where_the_key_was_looks_at_the_shelf() {
    let mut session = in_the_lit_closet();
    double_click(&mut session, point("closet", "key"));
    settle(&mut session);
    click(&mut session, point("closet", "key"));
    assert_eq!(
        next_words(&mut session),
        "Shelves of clean glasses for the pub."
    );
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
fn a_slow_double_click_on_a_carried_item_never_shows_its_look() {
    let mut session = with_the_key();
    click(&mut session, SLOT_0);
    for _ in 0..10 {
        session.update(STEP);
        assert_eq!(session.player.speech(), None);
    }
    session.click(SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));
    for _ in 0..100 {
        session.update(STEP);
        assert_eq!(session.player.speech(), None);
    }
}

#[test]
fn the_key_opens_the_cupboard_and_is_swapped_for_a_coin() {
    let mut session = with_the_key();

    double_click(&mut session, SLOT_0);
    click(&mut session, point("hall", "cupboard"));
    assert_eq!(next_words(&mut session), "The key fits the cupboard!");
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["coin"]);
    assert_eq!(session.holding, None);
    assert_eq!(session.player.position(), stand("hall", "cupboard"));
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
    click(&mut session, point("hall", "cupboard"));
    assert_eq!(next_words(&mut session), "That doesn't work.");
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["brass_key", "coin"]);
}

fn in_the_closet() -> Session {
    let mut session = session();
    double_click(&mut session, point("hall", "door"));
    settle(&mut session);
    session
}

#[test]
fn a_double_click_on_the_door_opens_it_and_goes_into_the_closet() {
    let mut session = session();
    assert_eq!(session.room_id, "hall");
    double_click(&mut session, point("hall", "door"));
    settle(&mut session);
    for _ in 0..3 {
        session.update(STEP);
    }

    assert_eq!(session.room_id, "closet");
    assert!(session.take_room_changed());
    assert!(!session.take_room_changed(), "the change is reported once");
    assert_eq!(session.player.position(), stand("closet", "door"));
}

#[test]
fn the_closet_is_dark_until_the_switch_is_used() {
    let mut session = in_the_closet();
    assert!(session.is_dark());
    assert_eq!(session.world.frame("closet_switch"), 0);

    click(&mut session, point("closet", "shelf"));
    assert_eq!(next_words(&mut session), "I can't see a thing in here.");
    settle(&mut session);

    double_click(&mut session, point("closet", "switch"));
    settle(&mut session);
    assert!(!session.is_dark());
    assert_eq!(session.world.frame("closet_switch"), 1);

    click(&mut session, point("closet", "shelf"));
    assert_eq!(
        next_words(&mut session),
        "Shelves of clean glasses for the pub."
    );
}

#[test]
fn the_switch_turns_the_light_off_again() {
    let mut session = in_the_closet();
    for expected_dark in [false, true] {
        double_click(&mut session, point("closet", "switch"));
        settle(&mut session);
        assert_eq!(session.is_dark(), expected_dark);
    }
}

#[test]
fn the_lights_stay_on_when_you_leave_and_come_back() {
    let mut session = in_the_closet();
    double_click(&mut session, point("closet", "switch"));
    settle(&mut session);

    double_click(&mut session, point("closet", "door"));
    settle(&mut session);
    session.update(STEP);
    assert_eq!(session.room_id, "hall");
    assert!(!session.is_dark(), "the hall is not a dark room");

    double_click(&mut session, point("hall", "door"));
    settle(&mut session);
    session.update(STEP);
    assert_eq!(session.room_id, "closet");
    assert!(!session.is_dark());
}

#[test]
fn the_closet_door_leads_back_to_the_bar_with_the_door_closed() {
    let mut session = in_the_closet();
    double_click(&mut session, point("closet", "door"));
    settle(&mut session);
    session.update(STEP);

    assert_eq!(session.room_id, "hall");
    assert_eq!(session.player.position(), stand("hall", "door"));
    assert_eq!(session.world.frame("door"), 0);
    assert_eq!(session.world.frame("closet_door"), 0);
}

#[test]
fn each_room_keeps_its_own_hotspot_handlers() {
    let mut session = in_the_closet();
    click(&mut session, point("closet", "door"));
    assert_eq!(next_words(&mut session), "The door back to the bar.");
    settle(&mut session);
    double_click(&mut session, point("closet", "door"));
    settle(&mut session);
    session.update(STEP);

    click(&mut session, point("hall", "door"));
    assert_eq!(next_words(&mut session), "The door to the cupboard.");
}

#[test]
fn a_script_sent_to_a_missing_room_leaves_the_game_where_it_was() {
    let mut session = session();
    let before = session.player.position();
    session
        .scripts
        .start("test", "enterRoom('nowhere', 10, 10)")
        .unwrap();
    settle(&mut session);
    session.update(STEP);

    assert_eq!(session.room_id, "hall");
    assert_eq!(session.player.position(), before);
    assert!(!session.take_room_changed());
    assert!(
        session
            .hotspot_at(ui::to_room(point("hall", "bar")))
            .is_some()
    );
}

#[test]
fn entering_a_room_at_an_unwalkable_spot_puts_the_player_on_the_nearest_walkable_one() {
    let mut session = session();
    session
        .scripts
        .start("test", "enterRoom('closet', 0, 0)")
        .unwrap();
    settle(&mut session);
    session.update(STEP);

    assert_eq!(session.room_id, "closet");
    let spot = session.player.position();
    assert_eq!(session.room.constrain(spot.x, spot.y), spot);
}

#[test]
fn a_lua_error_in_a_handler_ends_it_and_the_game_carries_on() {
    let mut session = session();
    session
        .scripts
        .load_definitions("test", "hotspot.bar.look = function() error('boom') end")
        .unwrap();
    click(&mut session, point("hall", "bar"));
    settle(&mut session);
    assert_eq!(session.player.position(), stand("hall", "bar"));

    click(&mut session, FLOOR);
    assert!(session.player.is_walking());
}

#[test]
fn a_script_that_errors_straight_away_is_dropped() {
    let mut session = session();
    session.scripts.start("test", "error('boom')").unwrap();
    session.update(STEP);
    assert!(!session.scripts.is_running());
}

#[test]
fn a_second_click_within_the_double_click_limits_interacts() {
    let bar = point("hall", "bar");
    let mut session = session();
    click(&mut session, bar);
    session.update(0.39);
    session.click(bar + vec2(3.9, 0.0));
    assert_eq!(next_words(&mut session), "Nobody is serving.");
}

#[test]
fn a_second_click_too_far_or_too_late_does_not() {
    let bar = point("hall", "bar");
    for (wait, offset) in [(0.39, 4.5), (0.41, 0.0)] {
        let mut session = session();
        click(&mut session, bar);
        session.update(wait);
        session.click(bar + vec2(offset, 0.0));
        assert_eq!(
            next_words(&mut session),
            "A long oak bar, ring-stained and well loved.",
            "wait {wait}, offset {offset}"
        );
    }
}

#[test]
fn a_double_click_on_an_item_while_another_is_held_does_not_swap_them() {
    let mut session = with_the_key();
    session.world.give("coin");
    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("brass_key"));

    double_click(&mut session, SLOT_1);
    assert_eq!(next_words(&mut session), "That doesn't work.");
    assert_eq!(session.holding, None);
}

#[test]
fn a_new_session_starts_from_scratch() {
    let mut first = with_the_key();
    first.world.set_light("closet", true);
    double_click(&mut first, SLOT_0);

    let second = Session::load().unwrap();
    assert!(second.world.items().is_empty());
    assert!(!second.world.is_lit("closet"));
    assert!(!second.world.is_hidden("key"));
    assert_eq!(second.holding, None);
    assert_eq!(second.room_id, "hall");
    assert!(second.scripts.is_running(), "the intro plays again");
}

#[test]
fn double_clicking_a_target_with_an_item_held_uses_the_item_without_cutting_it_short() {
    let mut session = with_the_key();
    double_click(&mut session, SLOT_0);
    double_click(&mut session, point("hall", "cupboard"));
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["coin"]);
}

#[test]
fn the_barman_trades_the_silver_coin_for_a_beer_and_goes_back_to_cleaning() {
    let mut session = session();
    session.world.give("coin");
    double_click(&mut session, SLOT_0);
    click(&mut session, point("hall", "barman"));
    settle(&mut session);

    assert_eq!(session.world.items(), vec!["beer"]);
    assert!(!session.world.is_animating("barman"));
    // back on the idle loop, which only ever shows the cleaning frames
    for _ in 0..200 {
        session.update(STEP);
        assert!(session.world.idle_frame("barman", 6, 4.0, 0.0) < 6);
    }
}

#[test]
fn the_barman_wants_paying_and_only_serves_once() {
    let mut session = session();
    double_click(&mut session, point("hall", "barman"));
    settle(&mut session);
    assert!(session.world.items().is_empty());

    session.world.give("coin");
    double_click(&mut session, SLOT_0);
    click(&mut session, point("hall", "barman"));
    settle(&mut session);
    double_click(&mut session, point("hall", "barman"));
    assert_eq!(next_words(&mut session), "One's plenty for now.");
}

#[test]
fn the_other_patrons_can_be_looked_at() {
    for id in ["drinker", "patron_left", "patron_right"] {
        let mut session = session();
        click(&mut session, point("hall", id));
        assert_ne!(next_words(&mut session), "I see nothing special.", "{id}");
    }
}

#[test]
fn a_double_click_on_a_door_goes_straight_through_it() {
    let mut session = session();
    double_click(&mut session, point("hall", "door"));
    assert_eq!(session.room_id, "closet");
    assert!(session.take_room_changed());
    assert_eq!(session.player.position(), stand("closet", "door"));
}

#[test]
fn a_double_click_on_the_floor_jumps_there() {
    let mut session = session();
    let target = vec2(60.0, 30.0);
    double_click(&mut session, vec2(target.x, target.y + PANEL_HEIGHT));
    assert_eq!(session.player.position(), target);
    assert!(!session.player.is_walking());
}

#[test]
fn double_clicking_a_beer_drinks_it_and_the_empty_glass_goes_back_to_the_barman() {
    let mut session = session();
    session.world.give("beer");
    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding, None);
    settle(&mut session);
    assert_eq!(session.world.items(), vec!["empty_glass"]);

    double_click(&mut session, SLOT_0);
    assert_eq!(session.holding.as_deref(), Some("empty_glass"));
    click(&mut session, point("hall", "barman"));
    let words = next_words(&mut session);
    assert!(!words.is_empty());
    settle(&mut session);
    assert!(session.world.items().is_empty());
}
