#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod automation;
mod effects;
mod render;

use std::path::Path;

use automation::{Automation, FIXED_DELTA, Frame, Inputs};
use effects::ScreenEffects;
use macroquad::prelude::*;
use render::{CharacterSprite, CursorSprite, FRAME_HEIGHT, RoomRenderer, Textures};
use wumpus_g_engine::manifest::Manifest;
use wumpus_g_engine::menu::{self, MenuAction, SaveMenu};
use wumpus_g_engine::save::{AUTOSAVE, SaveStore};
use wumpus_g_engine::session::Session;
use wumpus_g_engine::ui;
use wumpus_g_engine::ui::{RENDER_SCALE, SCREEN_HEIGHT, SCREEN_WIDTH};

const SPEECH_WRAP_WIDTH: f32 = 160.0;
const TEXT_SIZE: f32 = 8.0;
const LINE_SPACING: f32 = 1.15;
/// The cursor is drawn at this fraction of the virtual screen scale, so it is smaller than the art.
const CURSOR_SCALE: f32 = 0.5;
/// The held item is drawn at the cursor's bottom right, half the size of its 16x16 icon in the panel.
const HELD_ICON_SIZE: f32 = 8.0;
const SHOT_WIDTH: u32 = 1280;
const SHOT_HEIGHT: u32 = 720;

/// Where the virtual screen sits in the window: scaled to fit, centred, with bars at the edges.
struct View {
    scale: f32,
    offset: Vec2,
}

impl View {
    fn fit(width: f32, height: f32) -> Self {
        let scale = (width / SCREEN_WIDTH).min(height / SCREEN_HEIGHT);
        View {
            scale,
            offset: vec2(
                (width - SCREEN_WIDTH * scale) / 2.0,
                (height - SCREEN_HEIGHT * scale) / 2.0,
            ),
        }
    }

    fn current() -> Self {
        Self::fit(screen_width(), screen_height())
    }

    /// A virtual screen point (320x180, y up, the panel at the bottom) as window pixels.
    fn screen_to_window(&self, point: Vec2) -> Vec2 {
        vec2(
            self.offset.x + point.x * self.scale,
            self.offset.y + (SCREEN_HEIGHT - point.y) * self.scale,
        )
    }

    /// A point in the room (y up, drawn above the panel) as window pixels.
    fn room_to_window(&self, point: Vec2) -> Vec2 {
        self.screen_to_window(ui::to_screen(point))
    }

    /// Window pixels as a virtual screen point.
    fn window_to_screen(&self, window: Vec2) -> Vec2 {
        vec2(
            (window.x - self.offset.x) / self.scale,
            SCREEN_HEIGHT - (window.y - self.offset.y) / self.scale,
        )
    }
}

fn load_session() -> Session {
    Session::load().unwrap_or_else(|error| panic!("{error}"))
}

/// The previous game from the autosave, or a new one if there is none or it cannot be read.
fn resume_session(store: &SaveStore) -> Session {
    match store.list().first() {
        Some(entry) if entry.name == AUTOSAVE => store
            .read(AUTOSAVE)
            .and_then(|data| Session::from_save(&data))
            .unwrap_or_else(|error| {
                eprintln!("[save] starting a new game: {error}");
                load_session()
            }),
        _ => load_session(),
    }
}

struct Game {
    sprite: CharacterSprite,
    cursor: CursorSprite,
    /// Set by the first cursor input, so a scripted run shows no pointer until it moves one.
    cursor_placed: bool,
    effects: ScreenEffects,
    renderer: RoomRenderer,
    textures: Textures,
    session: Session,
    show_debug: bool,
    store: SaveStore,
    /// Scripted runs neither load nor write saves, so they play out the same way every time.
    persist: bool,
    menu: Option<SaveMenu>,
    quit: bool,
}

impl Game {
    fn new(new_game: bool, persist: bool) -> Self {
        let store = SaveStore::in_default_dir(
            &Manifest::load().map_or_else(|_| "WumpusGEngine".to_owned(), |manifest| manifest.id),
        );
        let session = match (persist, new_game) {
            (false, _) => load_session(),
            (true, true) => {
                if let Err(error) = store.archive_autosave() {
                    eprintln!("[save] {error}");
                }
                load_session()
            }
            (true, false) => resume_session(&store),
        };
        if persist {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |since| since.subsec_nanos() ^ since.as_secs() as u32);
            session.world.seed_street(seed);
        }
        let mut textures = Textures::preload(&session.manifest.rooms);
        Game {
            sprite: CharacterSprite::new(&session.manifest.player),
            cursor: CursorSprite::new(&session.manifest.cursor),
            cursor_placed: false,
            effects: ScreenEffects::default(),
            renderer: RoomRenderer::new(&session.room, &mut textures),
            textures,
            session,
            show_debug: false,
            store,
            persist,
            menu: None,
            quit: false,
        }
    }

    fn restart(&mut self) {
        self.session = load_session();
        // Loaded again so that edited art shows up in a debug build.
        self.textures = Textures::preload(&self.session.manifest.rooms);
        self.renderer = RoomRenderer::new(&self.session.room, &mut self.textures);
    }

    fn update(&mut self, inputs: &Inputs, delta: f32) {
        if let Some(cursor) = inputs.cursor {
            self.cursor_placed = true;
            self.session.move_cursor(cursor);
        }
        if inputs.toggle_menu {
            self.toggle_menu();
        }
        if inputs.escape {
            if self.menu.is_some() {
                self.menu = None;
            } else {
                self.quit = true;
            }
        }
        if self.menu.is_some() {
            self.update_menu(inputs);
            return;
        }
        if inputs.restart {
            self.restart();
        }
        if inputs.toggle_debug {
            self.show_debug = !self.show_debug;
        }
        if let Some(point) = inputs.click {
            self.session.click(point);
        }
        self.session.update(delta);
        if self.session.take_room_changed() {
            self.renderer = RoomRenderer::new(&self.session.room, &mut self.textures);
        }
    }

    fn toggle_menu(&mut self) {
        self.menu = match self.menu {
            Some(_) => None,
            None => Some(SaveMenu::new(self.store.list())),
        };
    }

    fn update_menu(&mut self, inputs: &Inputs) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        if inputs.wheel != 0.0 {
            menu.scroll(if inputs.wheel > 0.0 { -1 } else { 1 });
        }
        let Some(point) = inputs.click else {
            return;
        };
        match menu.click(point) {
            MenuAction::None => {}
            MenuAction::Close => self.menu = None,
            MenuAction::Save => {
                let result = self.store.write_new(&self.session.capture());
                menu.message = Some(match result {
                    Ok(name) => format!("Saved as {}", name.replace("save-", "Save ")),
                    Err(error) => {
                        eprintln!("[save] {error}");
                        "Could not save".to_owned()
                    }
                });
                menu.set_entries(self.store.list());
            }
            MenuAction::Load(name) => {
                match self
                    .store
                    .read(&name)
                    .and_then(|data| Session::from_save(&data))
                {
                    Ok(session) => {
                        let cursor = self.session.cursor;
                        self.session = session;
                        self.session.move_cursor(cursor);
                        self.renderer = RoomRenderer::new(&self.session.room, &mut self.textures);
                        self.menu = None;
                    }
                    Err(error) => {
                        eprintln!("[save] {error}");
                        menu.message = Some("Could not load that save".to_owned());
                    }
                }
            }
        }
    }

    /// Writes the autosave that the next start resumes from.
    fn save_on_exit(&self) {
        if !self.persist {
            return;
        }
        if let Err(error) = self.store.write_autosave(&self.session.capture()) {
            eprintln!("[save] {error}");
        }
    }

    /// Draws to the window, or to `output` (used for screenshots) when given.
    fn draw(&self, target: &RenderTarget, view: &View, output: Option<&RenderTarget>) {
        let camera = |top: f32| Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, top, SCREEN_WIDTH, SCREEN_HEIGHT))
        };

        // The room is drawn shifted up so that its bottom edge sits on top of the panel.
        set_camera(&camera(ui::PANEL_HEIGHT));
        clear_background(self.session.manifest.theme.background);
        self.renderer.draw(
            &self.session.player,
            &self.sprite,
            &self.session.world,
            self.session.is_dark(),
            self.session.manifest.theme.darkness,
        );
        set_camera(&camera(0.0));
        self.draw_panel();

        match output {
            Some(output) => set_camera(&Camera2D {
                render_target: Some(output.clone()),
                ..Camera2D::from_display_rect(Rect::new(
                    0.0,
                    0.0,
                    output.texture.width(),
                    output.texture.height(),
                ))
            }),
            None => set_default_camera(),
        }
        clear_background(self.session.manifest.theme.background);
        let effect = self.session.world.screen_effect();
        let applied = effect
            .as_ref()
            .is_some_and(|effect| self.effects.begin(effect));
        draw_texture_ex(
            &target.texture,
            view.offset.x,
            view.offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SCREEN_WIDTH * view.scale, SCREEN_HEIGHT * view.scale)),
                flip_y: true,
                ..Default::default()
            },
        );
        if applied {
            self.effects.end();
        }

        self.draw_speech(view);
        if self.show_debug {
            self.draw_debug(view);
        }
        if self.menu.is_some() {
            self.draw_menu(view);
        }
        if self.cursor_placed {
            let at = view.screen_to_window(self.session.cursor);
            self.cursor
                .draw(at, view.scale * CURSOR_SCALE, self.session.input_locked());
            self.draw_held_item(at, view);
        }
        set_default_camera();
    }

    /// The panel along the bottom: a box per carried item, drawn into the virtual screen.
    /// Items are squares of their colour until there is icon art.
    fn draw_panel(&self) {
        let session = &self.session;
        let top = |rect: Rect| SCREEN_HEIGHT - (rect.y + rect.h);
        draw_rectangle(
            0.0,
            SCREEN_HEIGHT - ui::PANEL_HEIGHT,
            SCREEN_WIDTH,
            ui::PANEL_HEIGHT,
            self.session.manifest.theme.panel,
        );

        for (index, id) in session
            .world
            .items()
            .iter()
            .take(ui::SLOT_COUNT)
            .enumerate()
        {
            let slot = ui::slot_rect(index);
            let shade = if session.holding.as_deref() == Some(id.as_str()) {
                Color::new(1.0, 0.9, 0.2, 0.4)
            } else if slot.contains(session.cursor) {
                Color::new(1.0, 1.0, 1.0, 0.25)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.1)
            };
            draw_rectangle(slot.x, top(slot), slot.w, slot.h, shade);
            let color = session.items.color(id);
            draw_rectangle(
                slot.x + 4.0,
                top(slot) + 2.0,
                slot.w - 8.0,
                slot.h - 4.0,
                color,
            );
        }
    }

    /// A small icon of the item being used, at the bottom right of the cursor (`cursor` is its window
    /// position). Items are squares of their colour until there is icon art.
    fn draw_held_item(&self, cursor: Vec2, view: &View) {
        let session = &self.session;
        let Some(id) = &session.holding else {
            return;
        };
        let color = session.items.color(id);
        let corner = cursor + self.cursor.size() * view.scale * CURSOR_SCALE;
        let size = HELD_ICON_SIZE * view.scale;
        draw_rectangle(corner.x.round(), corner.y.round(), size, size, color);
        draw_rectangle_lines(
            corner.x.round(),
            corner.y.round(),
            size,
            size,
            view.scale * 0.5,
            BLACK,
        );
    }

    /// F2: a dimmed screen with a box holding the buttons and the list of saves, at window resolution.
    fn draw_menu(&self, view: &View) {
        let Some(save_menu) = &self.menu else {
            return;
        };
        let to_window = |rect: Rect| {
            let top_left = view.screen_to_window(vec2(rect.x, rect.y + rect.h));
            Rect::new(
                top_left.x,
                top_left.y,
                rect.w * view.scale,
                rect.h * view.scale,
            )
        };
        let size = (TEXT_SIZE * view.scale).round().max(1.0) as u16;
        let ascent = measure_text("Ag", None, size, 1.0).offset_y;
        let label = |rect: Rect, text: &str, color: Color| {
            let inset = 3.0 * view.scale;
            let y = rect.y + (rect.h - size as f32) / 2.0 + ascent;
            draw_text(
                text,
                (rect.x + inset).round(),
                y.round(),
                size as f32,
                color,
            );
        };
        let button = |rect: Rect, text: &str, hot: bool| {
            let rect = to_window(rect);
            let shade = if hot { 0.28 } else { 0.14 };
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(1.0, 1.0, 1.0, shade),
            );
            label(rect, text, WHITE);
        };
        let cursor = self.session.cursor;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        let panel = to_window(menu::PANEL);
        let border = view.scale.round().max(1.0);
        draw_rectangle(
            panel.x - border,
            panel.y - border,
            panel.w + border * 2.0,
            panel.h + border * 2.0,
            Color::new(0.95, 0.9, 0.7, 1.0),
        );
        draw_rectangle(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            Color::new(0.06, 0.05, 0.1, 1.0),
        );
        label(
            Rect::new(
                panel.x,
                panel.y + 2.0 * view.scale,
                panel.w,
                10.0 * view.scale,
            ),
            "Save / Load",
            Color::new(0.95, 0.9, 0.7, 1.0),
        );

        button(
            menu::save_button(),
            "Save game",
            menu::save_button().contains(cursor),
        );
        button(
            menu::close_button(),
            "Close (F2)",
            menu::close_button().contains(cursor),
        );
        if let Some(message) = &save_menu.message {
            label(
                Rect::new(
                    panel.x,
                    panel.y + panel.h - 10.0 * view.scale,
                    panel.w,
                    10.0 * view.scale,
                ),
                message,
                Color::new(0.6, 0.9, 0.6, 1.0),
            );
        }
        if save_menu.entries().is_empty() {
            label(to_window(menu::row_rect(0)), "No saves yet.", GRAY);
        }
        for (row, entry) in save_menu.visible().iter().enumerate() {
            let rect = menu::row_rect(row);
            button(rect, &entry.label, rect.contains(cursor));
        }
    }

    /// Drawn at window resolution so the text stays crisp however the virtual screen is scaled.
    fn draw_speech(&self, view: &View) {
        let Some(text) = self.session.player.speech() else {
            return;
        };
        let size = (TEXT_SIZE * view.scale).round().max(1.0) as u16;
        let lines = wrap_text(text, SPEECH_WRAP_WIDTH * view.scale, size);
        let line_height = size as f32 * LINE_SPACING;
        let ascent = measure_text("Ag", None, size, 1.0).offset_y;

        let feet = self.session.player.position();
        let left = (feet.x - SPEECH_WRAP_WIDTH / 2.0).clamp(0.0, SCREEN_WIDTH - SPEECH_WRAP_WIDTH);
        let block_height = lines.len() as f32 * line_height / view.scale;
        let visible_top = SCREEN_HEIGHT - ui::PANEL_HEIGHT;
        let top = (feet.y + FRAME_HEIGHT + 4.0 + block_height).min(visible_top);
        let origin = view.room_to_window(vec2(left, top));

        let pad = 3.0 * view.scale;
        let widest = lines
            .iter()
            .map(|line| measure_text(line, None, size, 1.0).width)
            .fold(0.0_f32, f32::max);
        let box_x = origin.x + (SPEECH_WRAP_WIDTH * view.scale - widest) / 2.0 - pad;
        let box_w = widest + pad * 2.0;
        let box_h = lines.len() as f32 * line_height + pad * 2.0 - (line_height - size as f32);
        let box_y = origin.y - pad;
        let border = view.scale.round().max(1.0);
        draw_rectangle(
            (box_x - border).round(),
            (box_y - border).round(),
            (box_w + border * 2.0).round(),
            (box_h + border * 2.0).round(),
            Color::new(0.95, 0.9, 0.7, 1.0),
        );
        draw_rectangle(
            box_x.round(),
            box_y.round(),
            box_w.round(),
            box_h.round(),
            Color::new(0.06, 0.05, 0.1, 0.92),
        );

        for (i, line) in lines.iter().enumerate() {
            let width = measure_text(line, None, size, 1.0).width;
            let x = origin.x + (SPEECH_WRAP_WIDTH * view.scale - width) / 2.0;
            draw_text(
                line,
                x.round(),
                (origin.y + i as f32 * line_height + ascent).round(),
                size as f32,
                WHITE,
            );
        }
    }

    /// F1: the authored walk area in green, the margin-adjusted area characters walk on in yellow.
    fn draw_debug(&self, view: &View) {
        let room = &self.session.room;
        let outline = |ring: &[Vec2], color: Color| {
            for (i, &a) in ring.iter().enumerate() {
                let (a, b) = (
                    view.room_to_window(a),
                    view.room_to_window(ring[(i + 1) % ring.len()]),
                );
                draw_line(a.x, a.y, b.x, b.y, 1.0, color);
            }
        };
        outline(room.outline(), GREEN);
        room.obstacles()
            .iter()
            .for_each(|ring| outline(ring, GREEN));
        outline(room.walk_outline(), YELLOW);
        room.walk_obstacles()
            .iter()
            .for_each(|ring| outline(ring, YELLOW));
    }

    fn print_state(&self) {
        let session = &self.session;
        let position = session.player.position();
        println!(
            "[state] room={} player at ({:.1}, {:.1}) facing {:?} holding={:?} carrying={:?} walking={} speaking={} scripts_running={}",
            session.room_id,
            position.x,
            position.y,
            session.player.facing(),
            session.holding,
            session.world.items(),
            session.player.is_walking(),
            session.player.is_speaking(),
            session.scripts.is_running(),
        );
    }
}

fn wrap_text(text: &str, max_width: f32, size: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && measure_text(&candidate, None, size, 1.0).width > max_width {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn real_inputs(view: &View) -> Inputs {
    let cursor = view.window_to_screen(Vec2::from(mouse_position()));
    Inputs {
        cursor: Some(cursor),
        click: is_mouse_button_pressed(MouseButton::Left).then_some(cursor),
        restart: is_key_pressed(KeyCode::R),
        toggle_debug: is_key_pressed(KeyCode::F1),
        toggle_menu: is_key_pressed(KeyCode::F2),
        escape: is_key_pressed(KeyCode::Escape),
        wheel: mouse_wheel().1,
    }
}

/// Renders the current frame at a fixed size into `output` and saves it, so a screenshot does not
/// depend on the window's size, position or visibility.
fn save_screenshot(game: &Game, target: &RenderTarget, output: &RenderTarget, path: &Path) {
    game.draw(
        target,
        &View::fit(SHOT_WIDTH as f32, SHOT_HEIGHT as f32),
        Some(output),
    );
    let image = output.texture.get_texture_data();
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)
            .unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
    }
    image.export_png(&path.to_string_lossy());
    println!("[shot] {}", path.display());
}

struct Args {
    script: Option<String>,
    out: String,
    /// Start a new game instead of resuming the autosave (existing saves are kept).
    new_game: bool,
    /// Play the game in this folder instead of the bundled one.
    game: Option<String>,
}

fn parse_args() -> Args {
    let mut args = Args {
        script: None,
        out: "screenshots".to_owned(),
        new_game: false,
        game: None,
    };
    let mut words = std::env::args().skip(1);
    while let Some(word) = words.next() {
        match word.as_str() {
            "--auto" => args.script = words.next(),
            "--out" => args.out = words.next().unwrap_or(args.out),
            "--game" => args.game = words.next(),
            "--new" | "-n" => args.new_game = true,
            "--fullscreen" | "-f" => {}
            other => eprintln!("ignoring unknown argument {other}"),
        }
    }
    args
}

/// `--check [dir]`: lists the problems in a game (the bundled one without a folder) and exits, before
/// any window opens.
fn check_and_exit(dir: Option<String>) -> ! {
    if let Some(dir) = dir {
        wumpus_g_engine::assets::use_dir(dir);
    }
    let problems = wumpus_g_engine::check::check_game();
    for problem in &problems {
        println!("{problem}");
    }
    println!("{} problem(s)", problems.len());
    std::process::exit(i32::from(!problems.is_empty()))
}

fn window_conf() -> Conf {
    let mut words = std::env::args();
    while let Some(word) = words.next() {
        if word == "--check" {
            check_and_exit(words.next());
        }
        if word == "--game"
            && let Some(dir) = words.next()
        {
            wumpus_g_engine::assets::use_dir(dir);
        }
    }
    Conf {
        window_title: Manifest::load().map_or_else(|_| "WumpusGEngine".to_owned(), |m| m.title),
        window_width: 1280,
        window_height: 720,
        fullscreen: std::env::args().any(|word| word == "--fullscreen" || word == "-f"),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = parse_args();
    if let Some(dir) = &args.game {
        wumpus_g_engine::assets::use_dir(dir);
    }
    let mut automation = args.script.as_deref().map(|script| {
        Automation::load(script, &args.out).unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2)
        })
    });

    let target = render_target(
        SCREEN_WIDTH as u32 * RENDER_SCALE,
        SCREEN_HEIGHT as u32 * RENDER_SCALE,
    );
    target.texture.set_filter(FilterMode::Nearest);
    let screenshot_target = automation
        .is_some()
        .then(|| render_target(SHOT_WIDTH, SHOT_HEIGHT));
    show_mouse(false);
    let mut game = Game::new(args.new_game, automation.is_none());
    prevent_quit();

    loop {
        let view = View::current();
        let (frame, delta) = match automation.as_mut() {
            Some(automation) => (automation.next_frame(FIXED_DELTA), FIXED_DELTA),
            None => (
                Frame {
                    inputs: real_inputs(&view),
                    ..Default::default()
                },
                get_frame_time(),
            ),
        };

        game.update(&frame.inputs, delta);
        if automation.is_none() && (game.quit || is_quit_requested()) {
            game.save_on_exit();
            break;
        }
        game.draw(&target, &view, None);

        if frame.print_state {
            game.print_state();
        }
        if let (Some(path), Some(output)) = (&frame.shot, &screenshot_target) {
            save_screenshot(&game, &target, output, path);
        }
        if frame.quit {
            std::process::exit(0);
        }
        next_frame().await;
    }
}
