mod automation;
mod render;

use std::path::Path;

use automation::{Automation, FIXED_DELTA, Frame, Inputs};
use game::actor::Actor;
use game::assets;
use game::room::{Hotspot, Room};
use game::script::ScriptRunner;
use game::ui::{self, Verb};
use macroquad::prelude::*;
use render::{CharacterSprite, FRAME_HEIGHT, RoomRenderer, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

const SPEECH_WRAP_WIDTH: f32 = 160.0;
const TEXT_SIZE: f32 = 8.0;
const LINE_SPACING: f32 = 1.15;
const INTRO_SCRIPT: &str = "scripts/intro.lua";
const ROOM_SCRIPT: &str = "scripts/hall.lua";
const START_ROOM: &str = "rooms/hall.lua";
const PLAYER_SPRITE: &str = "characters/player.png";
const CLEAR: Color = Color::new(0.1, 0.1, 0.2, 1.0);
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

    /// Game coordinates (y up) to window pixels.
    fn to_screen(&self, point: Vec2) -> Vec2 {
        vec2(
            self.offset.x + point.x * self.scale,
            self.offset.y + (VIRTUAL_HEIGHT - point.y) * self.scale,
        )
    }

    /// Window pixels to game coordinates (y up).
    fn to_game(&self, screen: Vec2) -> Vec2 {
        vec2(
            (screen.x - self.offset.x) / self.scale,
            VIRTUAL_HEIGHT - (screen.y - self.offset.y) / self.scale,
        )
    }
}

struct Game {
    sprite: CharacterSprite,
    room: Room,
    renderer: RoomRenderer,
    player: Actor,
    scripts: ScriptRunner,
    verb: Verb,
    cursor: Vec2,
    show_debug: bool,
}

impl Game {
    fn new() -> Self {
        let sprite = CharacterSprite::new(PLAYER_SPRITE);
        let (room, renderer, player, scripts) = Self::load_room();
        Game {
            sprite,
            room,
            renderer,
            player,
            scripts,
            verb: Verb::WalkTo,
            cursor: Vec2::ZERO,
            show_debug: false,
        }
    }

    fn load_room() -> (Room, RoomRenderer, Actor, ScriptRunner) {
        let source = assets::text(START_ROOM).unwrap_or_else(|| panic!("missing {START_ROOM}"));
        let room = Room::from_lua(&source, START_ROOM)
            .unwrap_or_else(|error| panic!("{START_ROOM}: {error}"));
        let renderer = RoomRenderer::new(&room);
        let start = room.constrain(60.0, 50.0);
        let player = Actor::new(start.x, start.y);

        let mut scripts = ScriptRunner::new().expect("Lua failed to start");
        let definitions =
            assets::text(ROOM_SCRIPT).unwrap_or_else(|| panic!("missing {ROOM_SCRIPT}"));
        scripts
            .load_definitions(ROOM_SCRIPT, &definitions)
            .unwrap_or_else(|error| panic!("{ROOM_SCRIPT}: {error}"));
        let intro = assets::text(INTRO_SCRIPT).unwrap_or_else(|| panic!("missing {INTRO_SCRIPT}"));
        scripts
            .start(INTRO_SCRIPT, &intro)
            .unwrap_or_else(|error| panic!("{INTRO_SCRIPT}: {error}"));
        (room, renderer, player, scripts)
    }

    fn restart(&mut self) {
        self.scripts.clear();
        (self.room, self.renderer, self.player, self.scripts) = Self::load_room();
        self.verb = Verb::WalkTo;
    }

    fn update(&mut self, inputs: &Inputs, delta: f32) {
        if inputs.restart {
            self.restart();
        }
        if inputs.toggle_debug {
            self.show_debug = !self.show_debug;
        }
        if let Some(cursor) = inputs.cursor {
            self.cursor = cursor;
        }
        if let Some(point) = inputs.click
            && !self.scripts.is_running()
        {
            self.click(point);
        }

        self.player.update(delta);
        self.scripts.update(delta, &self.room, &mut self.player);
    }

    /// A button picks the verb; anywhere else uses the verb on what is there, or just walks, and
    /// the verb goes back to "Walk to".
    fn click(&mut self, point: Vec2) {
        if let Some(verb) = ui::verb_at(point) {
            self.verb = verb;
            return;
        }
        if ui::in_panel(point) {
            return;
        }

        let verb = std::mem::replace(&mut self.verb, Verb::WalkTo);
        match self.room.hotspot_at(point) {
            Some(hotspot) => {
                let (id, walk_to) = (hotspot.id.clone(), hotspot.walk_to.or(Some(point)));
                self.scripts
                    .interact(&id, verb.key(), walk_to)
                    .unwrap_or_else(|error| eprintln!("[lua] {id}.{}: {error}", verb.key()));
            }
            None => self.player.walk_to(&self.room, point.x, point.y),
        }
    }

    fn hovered(&self) -> Option<&Hotspot> {
        if ui::in_panel(self.cursor) {
            None
        } else {
            self.room.hotspot_at(self.cursor)
        }
    }

    /// Draws to the window, or to `output` (used for screenshots) when given.
    fn draw(&self, target: &RenderTarget, view: &View, output: Option<&RenderTarget>) {
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT))
        });
        clear_background(CLEAR);
        self.renderer.draw(&self.player, &self.sprite);
        self.draw_verb_panel();

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

        self.draw_verb_text(view);
        self.draw_speech(view);
        if self.show_debug {
            self.draw_debug(view);
        }
        set_default_camera();
    }

    /// The bar's background and button boxes, drawn into the virtual screen.
    fn draw_verb_panel(&self) {
        let top_left = |rect: Rect| VIRTUAL_HEIGHT - (rect.y + rect.h);
        draw_rectangle(
            0.0,
            0.0,
            VIRTUAL_WIDTH,
            VIRTUAL_HEIGHT - ui::PANEL_BOTTOM,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        for verb in Verb::ALL {
            let rect = ui::button_rect(verb);
            let shade = if verb == self.verb {
                Color::new(1.0, 0.9, 0.2, 0.3)
            } else if rect.contains(self.cursor) {
                Color::new(1.0, 1.0, 1.0, 0.25)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.1)
            };
            draw_rectangle(rect.x, top_left(rect), rect.w, rect.h, shade);
        }
    }

    /// The labels and the sentence, drawn at window resolution so they stay crisp.
    fn draw_verb_text(&self, view: &View) {
        let size = (TEXT_SIZE * view.scale).round().max(1.0) as u16;
        let ascent = measure_text("Ag", None, size, 1.0).offset_y;
        let centred = |text: &str, left: f32, width: f32| {
            left + (width - measure_text(text, None, size, 1.0).width) / 2.0
        };

        for verb in Verb::ALL {
            let rect = ui::button_rect(verb);
            let corner = view.to_screen(vec2(rect.x, rect.y + rect.h));
            let color = if verb == self.verb {
                YELLOW
            } else if rect.contains(self.cursor) {
                WHITE
            } else {
                LIGHTGRAY
            };
            draw_text(
                verb.label(),
                centred(verb.label(), corner.x, rect.w * view.scale).round(),
                (corner.y + (rect.h * view.scale + ascent) / 2.0).round(),
                size as f32,
                color,
            );
        }

        let sentence = ui::sentence(self.verb, self.hovered());
        let baseline = view.to_screen(vec2(0.0, ui::SENTENCE_BASELINE));
        draw_text(
            &sentence,
            centred(&sentence, view.offset.x, VIRTUAL_WIDTH * view.scale).round(),
            baseline.y.round(),
            size as f32,
            WHITE,
        );
    }

    /// Drawn at window resolution so the text stays crisp however the virtual screen is scaled.
    fn draw_speech(&self, view: &View) {
        let Some(text) = self.player.speech() else {
            return;
        };
        let size = (TEXT_SIZE * view.scale).round().max(1.0) as u16;
        let lines = wrap_text(text, SPEECH_WRAP_WIDTH * view.scale, size);
        let line_height = size as f32 * LINE_SPACING;
        let ascent = measure_text("Ag", None, size, 1.0).offset_y;

        let feet = self.player.position();
        let left = (feet.x - SPEECH_WRAP_WIDTH / 2.0).clamp(0.0, VIRTUAL_WIDTH - SPEECH_WRAP_WIDTH);
        let block_height = lines.len() as f32 * line_height / view.scale;
        let top = (feet.y + FRAME_HEIGHT + 4.0 + block_height).min(ui::PANEL_BOTTOM);
        let origin = view.to_screen(vec2(left, top));

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
        let outline = |ring: &[Vec2], color: Color| {
            for (i, &a) in ring.iter().enumerate() {
                let (a, b) = (
                    view.to_screen(a),
                    view.to_screen(ring[(i + 1) % ring.len()]),
                );
                draw_line(a.x, a.y, b.x, b.y, 1.0, color);
            }
        };
        outline(self.room.outline(), GREEN);
        self.room
            .obstacles()
            .iter()
            .for_each(|ring| outline(ring, GREEN));
        outline(self.room.walk_outline(), YELLOW);
        self.room
            .walk_obstacles()
            .iter()
            .for_each(|ring| outline(ring, YELLOW));
    }

    fn print_state(&self) {
        let position = self.player.position();
        println!(
            "[state] player at ({:.1}, {:.1}) facing {:?} verb={:?} walking={} speaking={} scripts_running={}",
            position.x,
            position.y,
            self.player.facing(),
            self.verb,
            self.player.is_walking(),
            self.player.is_speaking(),
            self.scripts.is_running(),
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
    Inputs {
        cursor: Some(view.to_game(Vec2::from(mouse_position()))),
        click: is_mouse_button_pressed(MouseButton::Left)
            .then(|| view.to_game(Vec2::from(mouse_position()))),
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
