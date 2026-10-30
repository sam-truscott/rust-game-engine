mod automation;
mod render;

use std::path::Path;

use automation::{Automation, FIXED_DELTA, Frame, Inputs};
use game::session::Session;
use game::ui;
use macroquad::prelude::*;
use render::{CharacterSprite, FRAME_HEIGHT, RoomRenderer, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

const SPEECH_WRAP_WIDTH: f32 = 160.0;
const TEXT_SIZE: f32 = 8.0;
const LINE_SPACING: f32 = 1.15;
const PLAYER_SPRITE: &str = "characters/player.png";
const CLEAR: Color = Color::new(0.1, 0.1, 0.2, 1.0);
const PANEL_COLOR: Color = Color::new(0.06, 0.06, 0.1, 1.0);
const SHOT_WIDTH: u32 = 1280;
const SHOT_HEIGHT: u32 = 720;

/// Where the virtual screen sits in the window: scaled to fit, centred, with bars at the edges.
struct View {
    scale: f32,
    offset: Vec2,
}

impl View {
    fn fit(width: f32, height: f32) -> Self {
        let scale = (width / VIRTUAL_WIDTH).min(height / VIRTUAL_HEIGHT);
        View {
            scale,
            offset: vec2(
                (width - VIRTUAL_WIDTH * scale) / 2.0,
                (height - VIRTUAL_HEIGHT * scale) / 2.0,
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
            self.offset.y + (VIRTUAL_HEIGHT - point.y) * self.scale,
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
            VIRTUAL_HEIGHT - (window.y - self.offset.y) / self.scale,
        )
    }
}

fn load_session() -> Session {
    Session::load().unwrap_or_else(|error| panic!("{error}"))
}

struct Game {
    sprite: CharacterSprite,
    renderer: RoomRenderer,
    session: Session,
    show_debug: bool,
}

impl Game {
    fn new() -> Self {
        let session = load_session();
        Game {
            sprite: CharacterSprite::new(PLAYER_SPRITE),
            renderer: RoomRenderer::new(&session.room),
            session,
            show_debug: false,
        }
    }

    fn restart(&mut self) {
        self.session = load_session();
        self.renderer = RoomRenderer::new(&self.session.room);
    }

    fn update(&mut self, inputs: &Inputs, delta: f32) {
        if inputs.restart {
            self.restart();
        }
        if inputs.toggle_debug {
            self.show_debug = !self.show_debug;
        }
        if let Some(cursor) = inputs.cursor {
            self.session.move_cursor(cursor);
        }
        if let Some(point) = inputs.click {
            self.session.click(point);
        }
        self.session.update(delta);
    }

    /// Draws to the window, or to `output` (used for screenshots) when given.
    fn draw(&self, target: &RenderTarget, view: &View, output: Option<&RenderTarget>) {
        let camera = |top: f32| Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, top, VIRTUAL_WIDTH, VIRTUAL_HEIGHT))
        };

        // The room is drawn shifted up so that its bottom edge sits on top of the panel.
        set_camera(&camera(ui::PANEL_HEIGHT));
        clear_background(CLEAR);
        self.renderer
            .draw(&self.session.player, &self.sprite, &self.session.world);
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
        clear_background(CLEAR);
        draw_texture_ex(
            &target.texture,
            view.offset.x,
            view.offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    VIRTUAL_WIDTH * view.scale,
                    VIRTUAL_HEIGHT * view.scale,
                )),
                flip_y: true,
                ..Default::default()
            },
        );

        self.draw_speech(view);
        if self.show_debug {
            self.draw_debug(view);
        }
        set_default_camera();
    }

    /// The panel along the bottom: a box per carried item, drawn into the virtual screen.
    /// Items are squares of their colour until there is icon art.
    fn draw_panel(&self) {
        let session = &self.session;
        let top = |rect: Rect| VIRTUAL_HEIGHT - (rect.y + rect.h);
        draw_rectangle(
            0.0,
            VIRTUAL_HEIGHT - ui::PANEL_HEIGHT,
            VIRTUAL_WIDTH,
            ui::PANEL_HEIGHT,
            PANEL_COLOR,
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
            let color = session.items.get(id).map_or(GRAY, |item| item.color);
            draw_rectangle(
                slot.x + 4.0,
                top(slot) + 2.0,
                slot.w - 8.0,
                slot.h - 4.0,
                color,
            );
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
        let left = (feet.x - SPEECH_WRAP_WIDTH / 2.0).clamp(0.0, VIRTUAL_WIDTH - SPEECH_WRAP_WIDTH);
        let block_height = lines.len() as f32 * line_height / view.scale;
        let visible_top = VIRTUAL_HEIGHT - ui::PANEL_HEIGHT;
        let top = (feet.y + FRAME_HEIGHT + 4.0 + block_height).min(visible_top);
        let origin = view.room_to_window(vec2(left, top));

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
            "[state] player at ({:.1}, {:.1}) facing {:?} holding={:?} carrying={:?} walking={} speaking={} scripts_running={}",
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
}

fn parse_args() -> Args {
    let mut args = Args {
        script: None,
        out: "screenshots".to_owned(),
    };
    let mut words = std::env::args().skip(1);
    while let Some(word) = words.next() {
        match word.as_str() {
            "--auto" => args.script = words.next(),
            "--out" => args.out = words.next().unwrap_or(args.out),
            other => eprintln!("ignoring unknown argument {other}"),
        }
    }
    args
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Game".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = parse_args();
    let mut automation = args.script.as_deref().map(|script| {
        Automation::load(script, &args.out).unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2)
        })
    });

    let target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let screenshot_target = automation
        .is_some()
        .then(|| render_target(SHOT_WIDTH, SHOT_HEIGHT));
    let mut game = Game::new();

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
