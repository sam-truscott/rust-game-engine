use game::room::Room;
use game::ui::{self, Verb};
use macroquad::math::{Rect, vec2};

fn centre(rect: Rect) -> macroquad::math::Vec2 {
    vec2(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
}

#[test]
fn every_button_is_found_at_its_centre() {
    for verb in Verb::ALL {
        assert_eq!(ui::verb_at(centre(ui::button_rect(verb))), Some(verb));
    }
}

#[test]
fn no_button_in_the_gaps_or_below_the_bar() {
    let first = ui::button_rect(Verb::ALL[0]);
    let second = ui::button_rect(Verb::ALL[1]);
    let gap = vec2((first.x + first.w + second.x) / 2.0, centre(first).y);
    assert_eq!(ui::verb_at(gap), None);
    assert_eq!(ui::verb_at(vec2(centre(first).x, 50.0)), None);
}

#[test]
fn the_buttons_fit_on_screen_inside_the_bar() {
    for verb in Verb::ALL {
        let rect = ui::button_rect(verb);
        assert!(
            rect.x >= 0.0 && rect.x + rect.w <= ui::SCREEN_WIDTH,
            "{verb:?} is off screen"
        );
        assert!(
            rect.y >= ui::PANEL_BOTTOM && rect.y + rect.h <= ui::SCREEN_HEIGHT,
            "{verb:?} is outside the bar"
        );
    }
    const {
        assert!(
            ui::SENTENCE_BASELINE > ui::PANEL_BOTTOM && ui::SENTENCE_BASELINE < ui::SCREEN_HEIGHT
        )
    };
}

#[test]
fn the_bar_covers_the_top_of_the_screen_only() {
    assert!(ui::in_panel(vec2(100.0, ui::PANEL_BOTTOM)));
    assert!(ui::in_panel(vec2(100.0, 179.0)));
    assert!(!ui::in_panel(vec2(100.0, ui::PANEL_BOTTOM - 1.0)));
}

#[test]
fn verbs_have_distinct_labels_and_script_keys() {
    let mut labels: Vec<_> = Verb::ALL.iter().map(|verb| verb.label()).collect();
    let mut keys: Vec<_> = Verb::ALL.iter().map(|verb| verb.key()).collect();
    labels.sort();
    labels.dedup();
    keys.sort();
    keys.dedup();
    assert_eq!(labels.len(), Verb::ALL.len());
    assert_eq!(keys.len(), Verb::ALL.len());
}

#[test]
fn the_sentence_names_what_the_cursor_is_over() {
    assert_eq!(ui::sentence(Verb::LookAt, None, None), "Look at");
    assert_eq!(
        ui::sentence(Verb::LookAt, None, Some("counter")),
        "Look at counter"
    );
    assert_eq!(
        ui::sentence(Verb::WalkTo, None, Some("counter")),
        "Walk to counter"
    );
}

#[test]
fn holding_an_item_makes_the_sentence_use_it_with_something() {
    assert_eq!(
        ui::sentence(Verb::Use, Some("brass key"), None),
        "Use brass key with"
    );
    assert_eq!(
        ui::sentence(Verb::Use, Some("brass key"), Some("counter")),
        "Use brass key with counter"
    );
}

#[test]
fn inventory_slots_are_found_only_where_something_is_carried() {
    for index in 0..3 {
        assert_eq!(
            ui::slot_at(centre(ui::slot_rect(index)), 3),
            Some(index),
            "slot {index}"
        );
    }
    assert_eq!(ui::slot_at(centre(ui::slot_rect(3)), 3), None);
    assert_eq!(ui::slot_at(centre(ui::slot_rect(0)), 0), None);
    let gap = vec2(
        ui::slot_rect(0).x + ui::slot_rect(0).w + 1.0,
        centre(ui::slot_rect(0)).y,
    );
    assert_eq!(ui::slot_at(gap, 3), None);
}

#[test]
fn the_slots_fit_on_screen_inside_the_strip() {
    for index in 0..ui::SLOT_COUNT {
        let slot = ui::slot_rect(index);
        assert!(
            slot.x >= 0.0 && slot.x + slot.w <= ui::SCREEN_WIDTH,
            "slot {index}"
        );
        assert!(
            slot.y >= 0.0 && slot.y + slot.h <= ui::INVENTORY_TOP,
            "slot {index}"
        );
        assert!(ui::in_inventory(centre(slot)));
    }
    assert!(!ui::in_inventory(vec2(100.0, ui::INVENTORY_TOP)));
    assert!(!ui::in_inventory(vec2(100.0, ui::PANEL_BOTTOM)));
}

#[test]
fn hotspots_are_read_from_the_room_file() {
    let room = Room::from_lua(
        "return { walkArea = { {0,0}, {100,0}, {100,100}, {0,100} },
                 hotspots = {
                   { id = 'rug', name = 'rug', rect = { 10, 10, 50, 50 } },
                   { id = 'lamp', name = 'lamp', rect = { 40, 40, 10, 30 }, walkTo = { 45, 30 } },
                 } }",
        "test",
    )
    .unwrap();

    assert_eq!(room.hotspots().len(), 2);
    assert_eq!(room.hotspots()[0].walk_to, None);
    assert_eq!(room.hotspots()[1].walk_to, Some(vec2(45.0, 30.0)));
    assert_eq!(room.hotspot_at(vec2(15.0, 15.0)).unwrap().id, "rug");
    assert!(room.hotspot_at(vec2(90.0, 90.0)).is_none());
}

#[test]
fn where_hotspots_overlap_the_later_one_wins() {
    let room = Room::from_lua(
        "return { walkArea = { {0,0}, {100,0}, {100,100}, {0,100} },
                 hotspots = {
                   { id = 'rug', name = 'rug', rect = { 10, 10, 50, 50 } },
                   { id = 'lamp', name = 'lamp', rect = { 40, 40, 10, 30 } },
                 } }",
        "test",
    )
    .unwrap();
    assert_eq!(room.hotspot_at(vec2(45.0, 45.0)).unwrap().id, "lamp");
    assert_eq!(room.hotspot_at(vec2(20.0, 20.0)).unwrap().id, "rug");
}

#[test]
fn a_hotspot_missing_its_name_is_an_error() {
    assert!(
        Room::from_lua(
            "return { walkArea = { {0,0}, {10,0}, {10,10} }, hotspots = { { id = 'x', rect = { 0, 0, 1, 1 } } } }",
            "test",
        )
        .is_err()
    );
}
