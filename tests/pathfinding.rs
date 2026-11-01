use macroquad::math::{Vec2, vec2};
use wumpus_g_engine::pathfinder::is_visible;
use wumpus_g_engine::room::{Room, offset};

mod common;

use common::{assert_close, floor_outline as floor, ring};

fn block() -> Vec<Vec2> {
    ring(&[(80.0, 30.0), (120.0, 30.0), (120.0, 70.0), (80.0, 70.0)])
}

fn l_shape() -> Vec<Vec2> {
    ring(&[
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 40.0),
        (40.0, 40.0),
        (40.0, 100.0),
        (0.0, 100.0),
    ])
}

fn length(start: Vec2, path: &[Vec2]) -> f32 {
    let mut total = 0.0;
    let mut previous = start;
    for &point in path {
        total += previous.distance(point);
        previous = point;
    }
    total
}

fn assert_walkable(room: &Room, start: Vec2, path: &[Vec2]) {
    let mut previous = start;
    for &point in path {
        assert!(is_visible(room, previous, point), "{previous} -> {point}");
        previous = point;
    }
}

fn assert_rings_close(actual: &[Vec2], expected: &[(f32, f32)]) {
    assert_eq!(actual.len(), expected.len());
    for (point, &(x, y)) in actual.iter().zip(expected) {
        assert_close(point.x, x);
        assert_close(point.y, y);
    }
}

#[test]
fn clear_line_goes_straight_to_the_goal() {
    let room = Room::new(floor(), vec![]);
    let path = room.find_path(vec2(10.0, 50.0), 150.0, 20.0);
    assert_eq!(path, vec![vec2(150.0, 20.0)]);
}

#[test]
fn goes_around_an_obstacle() {
    let room = Room::new(floor(), vec![block()]);
    let start = vec2(20.0, 50.0);
    let path = room.find_path(start, 180.0, 50.0);

    assert!(path.len() > 1);
    assert_eq!(*path.last().unwrap(), vec2(180.0, 50.0));
    assert_walkable(&room, start, &path);
    assert!(length(start, &path) > 160.0);
    assert!(length(start, &path) < 200.0);
}

#[test]
fn obstacle_corners_on_the_line_are_not_cut_through() {
    // The straight line from corner to opposite corner passes through the block's interior.
    let room = Room::new(floor(), vec![block()]);
    let start = vec2(80.0, 30.0);
    let path = room.find_path(start, 120.0, 70.0);
    assert_walkable(&room, start, &path);
    assert!(length(start, &path) > start.distance(vec2(120.0, 70.0)) + 1.0);
}

#[test]
fn routes_through_the_inner_corner_of_a_concave_room() {
    let room = Room::new(l_shape(), vec![]);
    let start = vec2(90.0, 10.0);
    let path = room.find_path(start, 10.0, 90.0);

    assert_eq!(path, vec![vec2(40.0, 40.0), vec2(10.0, 90.0)]);
    assert_walkable(&room, start, &path);
}

#[test]
fn target_inside_an_obstacle_is_moved_to_its_edge() {
    let room = Room::new(floor(), vec![block()]);
    let start = vec2(20.0, 50.0);
    let path = room.find_path(start, 100.0, 50.0);

    let end = *path.last().unwrap();
    assert!(room.is_walkable(end));
    assert!(!(end.x > 80.01 && end.x < 119.99 && end.y > 30.01 && end.y < 69.99));
    assert_walkable(&room, start, &path);
}

#[test]
fn margin_keeps_paths_clear_of_obstacles() {
    let room = Room::with_options(floor(), vec![block()], 5.0, None, vec![]);
    let start = vec2(20.0, 50.0);
    let path = room.find_path(start, 180.0, 50.0);

    let mut previous = start;
    for &point in &path {
        for step in 0..=100 {
            let sample = previous.lerp(point, step as f32 / 100.0);
            let inside_margin =
                sample.x > 75.01 && sample.x < 124.99 && sample.y > 25.01 && sample.y < 74.99;
            assert!(!inside_margin, "too close to the block at {sample}");
        }
        previous = point;
    }
    assert_close(previous.x, 180.0);
}

#[test]
fn margin_keeps_targets_clear_of_walls() {
    let room = Room::with_options(floor(), vec![], 5.0, None, vec![]);

    let east = room.find_path(vec2(50.0, 50.0), 300.0, 50.0);
    assert_close(east.last().unwrap().x, 195.0);

    let corner = room.find_path(vec2(50.0, 50.0), -20.0, -20.0);
    assert_close(corner.last().unwrap().x, 5.0);
    assert_close(corner.last().unwrap().y, 5.0);
}

#[test]
fn offset_moves_edges_outward_or_inward_whatever_the_winding() {
    let counter_clockwise = ring(&[(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)]);
    let clockwise = ring(&[(10.0, 10.0), (10.0, 20.0), (20.0, 20.0), (20.0, 10.0)]);

    assert_rings_close(
        &offset(&counter_clockwise, 2.0),
        &[(8.0, 8.0), (22.0, 8.0), (22.0, 22.0), (8.0, 22.0)],
    );
    assert_rings_close(
        &offset(&clockwise, 2.0),
        &[(8.0, 8.0), (8.0, 22.0), (22.0, 22.0), (22.0, 8.0)],
    );
    assert_rings_close(
        &offset(&counter_clockwise, -2.0),
        &[(12.0, 12.0), (18.0, 12.0), (18.0, 18.0), (12.0, 18.0)],
    );
}

#[test]
fn repeated_queries_give_the_same_path() {
    let room = Room::new(floor(), vec![block()]);
    let start = vec2(20.0, 50.0);
    assert_eq!(
        room.find_path(start, 180.0, 50.0),
        room.find_path(start, 180.0, 50.0)
    );
}

#[test]
fn target_outside_the_room_is_moved_to_its_edge() {
    let room = Room::new(floor(), vec![]);
    let path = room.find_path(vec2(10.0, 10.0), 300.0, 50.0);
    let end = *path.last().unwrap();
    assert_close(end.x, 200.0);
    assert_close(end.y, 50.0);
}
