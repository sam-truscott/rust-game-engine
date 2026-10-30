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
fn no_button_in_the_gaps_or_above_the_panel() {
    let first = ui::button_rect(Verb::ALL[0]);
    let second = ui::button_rect(Verb::ALL[1]);
    let gap = vec2((first.x + first.w + second.x) / 2.0, centre(first).y);
    assert_eq!(ui::verb_at(gap), None);
    assert_eq!(
        ui::verb_at(vec2(centre(first).x, ui::PANEL_HEIGHT + 10.0)),
        None
    );
}

#[test]
fn the_buttons_fit_on_screen_inside_the_panel() {
    for verb in Verb::ALL {
        let rect = ui::button_rect(verb);
        assert!(
            rect.x >= 0.0 && rect.x + rect.w <= ui::SCREEN_WIDTH,
            "{verb:?} is off screen"
        );
        assert!(
            rect.y >= 0.0 && rect.y + rect.h <= ui::PANEL_HEIGHT,
            "{verb:?} is outside the panel"
        );
        assert!(ui::in_panel(centre(rect)));
    }
}

#[test]
fn the_buttons_sit_above_the_inventory_slots() {
    let lowest_button = Verb::ALL
        .iter()
        .map(|&verb| ui::button_rect(verb).y)
        .fold(f32::MAX, f32::min);
    let highest_slot = (0..ui::SLOT_COUNT)
        .map(|index| ui::slot_rect(index).y + ui::slot_rect(index).h)
        .fold(0.0, f32::max);
    assert!(lowest_button > highest_slot, "buttons and slots overlap");
}

#[test]
fn the_panel_covers_the_bottom_of_the_screen_only() {
    assert!(ui::in_panel(vec2(100.0, 0.0)));
    assert!(ui::in_panel(vec2(100.0, ui::PANEL_HEIGHT - 0.5)));
    assert!(!ui::in_panel(vec2(100.0, ui::PANEL_HEIGHT)));
    assert!(!ui::in_panel(vec2(100.0, 179.0)));
}

#[test]
fn screen_and_room_points_convert_both_ways() {
    let room = vec2(120.0, 55.0);
    let screen = ui::to_screen(room);
    assert_eq!(screen, vec2(120.0, 55.0 + ui::PANEL_HEIGHT));
    assert_eq!(ui::to_room(screen), room);
    assert!(!ui::in_panel(ui::to_screen(vec2(0.0, 0.0))));
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
fn the_slots_fit_on_screen_inside_the_panel() {
    for index in 0..ui::SLOT_COUNT {
        let slot = ui::slot_rect(index);
        assert!(
            slot.x >= 0.0 && slot.x + slot.w <= ui::SCREEN_WIDTH,
            "slot {index}"
        );
        assert!(
            slot.y >= 0.0 && slot.y + slot.h <= ui::PANEL_HEIGHT,
            "slot {index}"
        );
        assert!(ui::in_panel(centre(slot)));
    }
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
