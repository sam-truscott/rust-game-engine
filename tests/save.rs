use game::menu::{self, MenuAction, SaveMenu};
use game::save::{AUTOSAVE, SaveData, SaveStore};
use game::session::Session;
use macroquad::math::vec2;

fn temp_store(name: &str) -> SaveStore {
    let dir = std::env::temp_dir().join(format!("game-save-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    SaveStore::new(dir)
}

fn played() -> SaveData {
    let mut session = Session::load().unwrap();
    session.scripts.clear();
    session.world.give("brass_key");
    session.world.hide("key");
    session.world.set_flag("served", true);
    session.world.set_light("closet", true);
    session.world.set_frame("door", 3);
    session.capture()
}

#[test]
fn save_data_survives_the_text_format() {
    let data = played();
    assert_eq!(SaveData::from_text(&data.to_text()).unwrap(), data);
}

#[test]
fn bad_save_text_is_refused() {
    assert!(SaveData::from_text("").is_err());
    assert!(SaveData::from_text("game-save 1\nroom hall\n").is_err());
    assert!(SaveData::from_text("game-save 1\nroom hall\nposition 1 nan\n").is_err());
    assert!(SaveData::from_text("game-save 1\nroom hall\nposition 1 2\nfacing up\n").is_err());
    assert!(SaveData::from_text("game-save 1\nroom hall\nposition 1 2\nwhat\n").is_err());
}

#[test]
fn a_loaded_session_matches_the_saved_one() {
    let data = played();
    let session = Session::from_save(&data).unwrap();
    assert_eq!(session.capture(), data);
    assert!(!session.scripts.is_running(), "the intro does not replay");
    assert!(session.world.flag("served"));
}

#[test]
fn a_save_in_another_room_loads_there() {
    let mut data = played();
    data.room = "closet".to_owned();
    data.x = 100.0;
    data.y = 30.0;
    let session = Session::from_save(&data).unwrap();
    assert_eq!(session.room_id, "closet");
    assert!(
        Session::from_save(&SaveData {
            room: "nowhere".to_owned(),
            ..data
        })
        .is_err()
    );
}

#[test]
fn the_store_keeps_an_autosave_and_numbered_saves() {
    let store = temp_store("store");
    let data = played();
    assert!(store.list().is_empty());
    store.write_autosave(&data).unwrap();
    assert_eq!(store.write_new(&data).unwrap(), "save-1");
    assert_eq!(store.write_new(&data).unwrap(), "save-2");
    let names: Vec<String> = store.list().into_iter().map(|e| e.name).collect();
    assert_eq!(names, [AUTOSAVE, "save-2", "save-1"]);
    assert_eq!(store.read("save-1").unwrap(), data);
    assert!(store.read("../save-1").is_err());
    let _ = std::fs::remove_dir_all(store.dir());
}

#[test]
fn archiving_the_autosave_keeps_it_as_a_numbered_save() {
    let store = temp_store("archive");
    let data = played();
    store.archive_autosave().unwrap();
    store.write_autosave(&data).unwrap();
    store.write_new(&data).unwrap();
    store.archive_autosave().unwrap();
    let names: Vec<String> = store.list().into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["save-2", "save-1"]);
    let _ = std::fs::remove_dir_all(store.dir());
}

#[test]
fn the_menu_maps_clicks_to_actions_and_scrolls() {
    let store = temp_store("menu");
    let data = played();
    for _ in 0..10 {
        store.write_new(&data).unwrap();
    }
    let mut menu = SaveMenu::new(store.list());
    let centre = |r: macroquad::math::Rect| vec2(r.x + r.w / 2.0, r.y + r.h / 2.0);
    assert_eq!(menu.click(centre(menu::save_button())), MenuAction::Save);
    assert_eq!(menu.click(centre(menu::close_button())), MenuAction::Close);
    assert_eq!(menu.click(vec2(1.0, 1.0)), MenuAction::None);
    assert_eq!(
        menu.click(centre(menu::row_rect(0))),
        MenuAction::Load("save-10".to_owned())
    );
    menu.scroll(100);
    assert_eq!(menu.visible().len(), menu::VISIBLE_ROWS);
    assert_eq!(
        menu.click(centre(menu::row_rect(menu::VISIBLE_ROWS - 1))),
        MenuAction::Load("save-1".to_owned())
    );
    menu.scroll(-100);
    assert_eq!(menu.visible()[0].name, "save-10");
    let _ = std::fs::remove_dir_all(store.dir());
}
