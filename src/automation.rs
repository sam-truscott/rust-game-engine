//! Scripted play, for checking the game without real input or screen capture:
//! `game --auto steps.txt [--out dir]`. One command per line, `#` starts a comment.
//!
//! ```text
//! wait 3.0        game-time seconds to let pass
//! move 160 110    put the cursor at a screen position (320x180, y up; the panel is the bottom 40
//!                 and the room is above it, so a room point (x, y) is at (x, y + 40)), to hover
//! click 256 112   left click at a screen position (also moves the cursor there)
//! key r           restart (r) or toggle the walk-area overlay (f1)
//! shot name       save <out>/name.png (1280x720)
//! where           print the player's position and state
//! quit            end the run (also implied at the end of the script)
//! ```

use std::collections::VecDeque;
use std::path::PathBuf;

use macroquad::math::{Vec2, vec2};

/// Automated runs use a fixed time step so they play out the same way every time.
pub const FIXED_DELTA: f32 = 1.0 / 60.0;

#[derive(Default)]
pub struct Inputs {
    /// Where the cursor is now, if it moved.
    pub cursor: Option<Vec2>,
    pub click: Option<Vec2>,
    pub restart: bool,
    pub toggle_debug: bool,
}

#[derive(Default)]
pub struct Frame {
    pub inputs: Inputs,
    pub shot: Option<PathBuf>,
    pub print_state: bool,
    pub quit: bool,
}

#[derive(Debug, PartialEq)]
enum Step {
    Wait(f32),
    Move(Vec2),
    Click(Vec2),
    Restart,
    ToggleDebug,
    Shot(String),
    Where,
    Quit,
}

pub struct Automation {
    steps: VecDeque<Step>,
    waiting: f32,
    out_dir: PathBuf,
}

impl Automation {
    pub fn load(path: &str, out_dir: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
        Ok(Automation {
            steps: parse(&text)?,
            waiting: 0.0,
            out_dir: out_dir.into(),
        })
    }

    /// What happens in the next frame; each step other than `wait` takes one frame.
    pub fn next_frame(&mut self, delta_seconds: f32) -> Frame {
        let mut frame = Frame::default();
        if self.waiting > 0.0 {
            self.waiting -= delta_seconds;
            return frame;
        }
        match self.steps.pop_front() {
            Some(Step::Wait(seconds)) => self.waiting = seconds,
            Some(Step::Move(position)) => frame.inputs.cursor = Some(position),
            Some(Step::Click(position)) => {
                frame.inputs.cursor = Some(position);
                frame.inputs.click = Some(position);
            }
            Some(Step::Restart) => frame.inputs.restart = true,
            Some(Step::ToggleDebug) => frame.inputs.toggle_debug = true,
            Some(Step::Shot(name)) => frame.shot = Some(self.out_dir.join(format!("{name}.png"))),
            Some(Step::Where) => frame.print_state = true,
            Some(Step::Quit) | None => frame.quit = true,
        }
        frame
    }
}

fn parse(text: &str) -> Result<VecDeque<Step>, String> {
    let mut steps = VecDeque::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default();
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        let fail = |message: &str| format!("line {}: {message}: {}", number + 1, line.trim());
        let number_at = |index: usize| -> Result<f32, String> {
            words
                .get(index)
                .and_then(|word| word.parse().ok())
                .ok_or_else(|| fail("expected a number"))
        };

        steps.push_back(match words[0] {
            "wait" => Step::Wait(number_at(1)?),
            "move" => Step::Move(vec2(number_at(1)?, number_at(2)?)),
            "click" => Step::Click(vec2(number_at(1)?, number_at(2)?)),
            "key" => match words.get(1).copied() {
                Some("r") => Step::Restart,
                Some("f1") => Step::ToggleDebug,
                _ => return Err(fail("key must be r or f1")),
            },
            "shot" => Step::Shot(
                words
                    .get(1)
                    .ok_or_else(|| fail("expected a name"))?
                    .to_string(),
            ),
            "where" => Step::Where,
            "quit" => Step::Quit,
            _ => return Err(fail("unknown command")),
        });
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_command_and_skips_comments() {
        let steps = parse(
            "# intro\nwait 1.5\n\nmove 5 6\nclick 10 20  # to the left\nkey r\nkey f1\nshot a\nwhere\nquit\n",
        )
        .unwrap();
        assert_eq!(
            steps,
            vec![
                Step::Wait(1.5),
                Step::Move(vec2(5.0, 6.0)),
                Step::Click(vec2(10.0, 20.0)),
                Step::Restart,
                Step::ToggleDebug,
                Step::Shot("a".to_owned()),
                Step::Where,
                Step::Quit,
            ]
        );
    }

    #[test]
    fn reports_the_line_of_a_bad_command() {
        assert!(
            parse("wait 1\nfly away\n")
                .unwrap_err()
                .starts_with("line 2:")
        );
        assert!(parse("click 5\n").is_err());
        assert!(parse("key x\n").is_err());
        assert!(parse("shot\n").is_err());
    }

    #[test]
    fn wait_holds_back_later_steps_for_that_much_game_time() {
        let mut automation = Automation {
            steps: parse("wait 0.5\nwhere\n").unwrap(),
            waiting: 0.0,
            out_dir: ".".into(),
        };
        let mut frames = 0;
        loop {
            frames += 1;
            if automation.next_frame(0.125).print_state {
                break;
            }
            assert!(frames < 100);
        }
        // one frame to start the wait, four of waiting (0.125 is exact in binary), one to run the next step
        assert_eq!(frames, 6);
    }

    #[test]
    fn the_run_ends_when_the_script_does() {
        let mut automation = Automation {
            steps: parse("where\n").unwrap(),
            waiting: 0.0,
            out_dir: ".".into(),
        };
        assert!(automation.next_frame(0.1).print_state);
        assert!(automation.next_frame(0.1).quit);
    }
}
