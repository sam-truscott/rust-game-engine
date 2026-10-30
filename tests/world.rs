use game::items::Items;
use game::world::World;

#[test]
fn items_are_carried_in_the_order_they_were_given_without_duplicates() {
    let world = World::default();
    world.give("key");
    world.give("coin");
    world.give("key");
    assert_eq!(world.items(), vec!["key", "coin"]);
    assert!(world.has("key"));
    assert!(!world.has("rope"));
}

#[test]
fn taking_an_item_removes_it_and_says_whether_it_was_there() {
    let world = World::default();
    world.give("key");
    assert!(world.take("key"));
    assert!(!world.take("key"));
    assert!(world.items().is_empty());
}

#[test]
fn hiding_is_remembered_and_only_for_that_id() {
    let world = World::default();
    world.hide("key");
    assert!(world.is_hidden("key"));
    assert!(!world.is_hidden("counter"));
}

#[test]
fn clones_share_the_same_state() {
    let world = World::default();
    let other = world.clone();
    other.give("key");
    other.hide("rug");
    assert!(world.has("key"));
    assert!(world.is_hidden("rug"));
}

#[test]
fn items_are_read_from_a_lua_file_in_a_stable_order() {
    let items = Items::from_lua(
        "return { zebra = { name = 'toy zebra' }, apple = { name = 'red apple', color = { 1, 0, 0 } } }",
        "test",
    )
    .unwrap();

    let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
    assert_eq!(ids, vec!["apple", "zebra"]);
    assert_eq!(items.name("apple"), "red apple");
    assert_eq!(items.get("apple").unwrap().color.r, 1.0);
    assert_eq!(items.get("zebra").unwrap().color.r, 0.7);
}

#[test]
fn an_unknown_item_is_shown_by_its_id() {
    let items = Items::from_lua("return {}", "test").unwrap();
    assert!(items.get("ghost").is_none());
    assert_eq!(items.name("ghost"), "ghost");
}

#[test]
fn an_item_without_a_name_is_an_error() {
    assert!(Items::from_lua("return { rope = { color = { 1, 1, 1 } } }", "test").is_err());
}

#[test]
fn an_animation_steps_through_its_frames_then_stays_on_the_last() {
    let world = World::default();
    assert_eq!(world.frame("door"), 0);
    world.animate("door", 0, 2, 10.0);
    assert!(world.is_animating("door"));
    assert_eq!(world.frame("door"), 0);
    world.advance(0.11);
    assert_eq!(world.frame("door"), 1);
    world.advance(0.1);
    assert_eq!(world.frame("door"), 2);
    world.advance(0.1);
    assert!(!world.is_animating("door"));
    assert_eq!(world.frame("door"), 2);
}

#[test]
fn an_animation_can_run_backwards() {
    let world = World::default();
    world.animate("door", 3, 0, 10.0);
    world.advance(0.15);
    assert_eq!(world.frame("door"), 2);
    world.advance(1.0);
    assert_eq!(world.frame("door"), 0);
}

#[test]
fn setting_a_frame_stops_an_animation_and_other_props_are_unaffected() {
    let world = World::default();
    world.animate("door", 0, 3, 10.0);
    world.set_frame("door", 1);
    assert!(!world.is_animating("door"));
    assert_eq!(world.frame("door"), 1);
    assert_eq!(world.frame("lid"), 0);
}
