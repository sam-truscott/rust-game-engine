use game::actor::Actor;
use game::direction::Direction;
use game::room::Room;
use macroquad::math::vec2;

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

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn starts_facing_the_viewer_and_standing() {
    let actor = Actor::new(50.0, 50.0);
    assert_eq!(actor.facing(), Direction::Front);
    assert!(!actor.is_walking());
    assert_eq!(actor.walk_distance(), 0.0);
}

#[test]
fn faces_the_direction_of_travel() {
    let room = floor();
    let mut actor = Actor::new(100.0, 50.0);

    actor.walk_to(&room, 150.0, 50.0);
    actor.update(0.1);
    assert_eq!(actor.facing(), Direction::Right);

    actor.walk_to(&room, 20.0, 50.0);
    actor.update(0.1);
    assert_eq!(actor.facing(), Direction::Left);

    actor.walk_to(&room, actor.position().x, 95.0);
    actor.update(0.1);
    assert_eq!(actor.facing(), Direction::Back);

    actor.walk_to(&room, actor.position().x, 5.0);
    actor.update(0.1);
    assert_eq!(actor.facing(), Direction::Front);
}

#[test]
fn mostly_vertical_diagonals_face_front_or_back() {
    let room = floor();
    let mut actor = Actor::new(100.0, 50.0);
    actor.walk_to(&room, 110.0, 10.0);
    actor.update(0.1);
    assert_eq!(actor.facing(), Direction::Front);
}

#[test]
fn keeps_facing_after_stopping() {
    let room = floor();
    let mut actor = Actor::new(50.0, 50.0);
    actor.walk_to(&room, 60.0, 50.0);
    actor.update(5.0);
    assert!(!actor.is_walking());
    assert_eq!(actor.facing(), Direction::Right);
}

#[test]
fn walk_distance_matches_distance_travelled() {
    let room = floor();
    let mut actor = Actor::new(50.0, 50.0);
    actor.walk_to(&room, 110.0, 50.0);
    actor.update(0.5);
    assert_close(actor.walk_distance(), 30.0);
    actor.update(5.0);
    assert!(!actor.is_walking());
    assert_close(actor.walk_distance(), 60.0);
}

#[test]
fn walk_distance_restarts_with_each_walk_but_not_when_redirected_mid_walk() {
    let room = floor();
    let mut actor = Actor::new(50.0, 50.0);
    actor.walk_to(&room, 110.0, 50.0);
    actor.update(5.0);
    assert_close(actor.walk_distance(), 60.0);

    actor.walk_to(&room, 60.0, 50.0);
    assert_close(actor.walk_distance(), 0.0);
    actor.update(0.25);
    assert_close(actor.walk_distance(), 15.0);

    actor.walk_to(&room, 110.0, 50.0);
    assert_close(actor.walk_distance(), 15.0);
}

#[test]
fn speech_lasts_longer_for_longer_text_and_then_ends() {
    let mut actor = Actor::new(50.0, 50.0);
    actor.say("Hi");
    assert_eq!(actor.speech(), Some("Hi"));
    actor.update(1.0);
    assert!(actor.is_speaking());
    actor.update(0.2);
    assert!(!actor.is_speaking());
}
