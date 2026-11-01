use macroquad::math::{Rect, vec2};
use wumpus_g_engine::room::Room;
use wumpus_g_engine::ui;

fn centre(rect: Rect) -> macroquad::math::Vec2 {
    vec2(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)
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
