//! The street outside a window: cars that drive past at random intervals.

use macroquad::color::Color;
use macroquad::math::Rect;
use mlua::Table;

/// How often cars come, each a (low, high) range of seconds picked at random.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timing {
    /// Between one car leaving and the next appearing.
    pub gap: (f32, f32),
    /// A car takes this long to cross the window.
    pub crossing: (f32, f32),
    /// Before the first car, which comes soon so a new game shows one.
    pub first: (f32, f32),
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            gap: (7.0, 25.0),
            crossing: (2.4, 3.8),
            first: (3.0, 8.0),
        }
    }
}

/// How the cars and their light look. The defaults are a dark saloon seen from the side, with warm
/// headlights; a pattern letter is B (body), C (cabin), H (headlight), T (taillight) or W (wheel),
/// facing right, and anything else is empty.
#[derive(Clone, Debug)]
pub struct CarLook {
    pub pattern: Vec<String>,
    /// A car picks one of these for its body.
    pub bodies: Vec<Color>,
    pub cabin: Color,
    pub tail: Color,
    pub wheel: Color,
    pub headlight: Color,
    /// The light a car throws down the wall and floor.
    pub beam: Color,
    /// The streak across the mirror.
    pub glint: Color,
    /// A faint tint over the whole room as a car passes, and how strong it gets (0-1).
    pub wash: Color,
    pub wash_strength: f32,
    /// Game units a pattern cell is wide.
    pub block: f32,
}

impl Default for CarLook {
    fn default() -> Self {
        CarLook {
            pattern: ["..BBBBB...", ".BCCBCCB..", "TBBBBBBBBH", ".WW....WW."]
                .map(str::to_owned)
                .to_vec(),
            bodies: vec![
                Color::new(0.10, 0.13, 0.20, 1.0),
                Color::new(0.30, 0.09, 0.10, 1.0),
                Color::new(0.12, 0.20, 0.17, 1.0),
            ],
            cabin: Color::new(0.20, 0.25, 0.38, 1.0),
            tail: Color::new(0.85, 0.12, 0.10, 1.0),
            wheel: Color::new(0.02, 0.02, 0.04, 1.0),
            headlight: Color::new(1.0, 0.86, 0.55, 1.0),
            beam: Color::new(1.0, 0.7, 0.35, 1.0),
            glint: Color::new(1.0, 0.9, 0.7, 1.0),
            wash: Color::new(1.0, 0.8, 0.5, 1.0),
            wash_strength: 0.05,
            block: 1.25,
        }
    }
}

/// A window onto a street: where cars are seen (`glass`, room coordinates, y up), an optional mirror
/// that catches their light and the y of the floor the light reaches. In a room file:
/// `street = { glass = {x,y,w,h}, mirror = {x,y,w,h}?, floor = y?, gap = {lo,hi}?, crossing = {lo,hi}?,
/// first = {lo,hi}?, car = { pattern = {"row",...}?, bodies = {{r,g,b},...}?, cabin?, tail?, wheel?,
/// headlight?, block? }?, light = { beam?, glint?, wash?, wash_strength? }? }`.
#[derive(Clone, Debug)]
pub struct StreetView {
    pub glass: Rect,
    pub mirror: Option<Rect>,
    pub floor: f32,
    pub timing: Timing,
    pub look: CarLook,
}

impl StreetView {
    pub fn from_lua(street: &Table) -> mlua::Result<Self> {
        let fail = |message: &str| mlua::Error::runtime(message.to_owned());
        let rect = |key: &str| -> mlua::Result<Option<Rect>> {
            street
                .get::<Option<Table>>(key)?
                .map(|r| Ok(Rect::new(r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
                .transpose()
        };
        let glass = rect("glass")?.ok_or_else(|| fail("street needs a glass rect"))?;

        let mut timing = Timing::default();
        for (key, slot) in [
            ("gap", &mut timing.gap),
            ("crossing", &mut timing.crossing),
            ("first", &mut timing.first),
        ] {
            if let Some(range) = street.get::<Option<Table>>(key)? {
                let (low, high): (f32, f32) = (range.get(1)?, range.get(2)?);
                if !(low.is_finite() && high.is_finite() && 0.0 < low && low <= high) {
                    return Err(fail(&format!("street.{key} needs 0 < low <= high")));
                }
                *slot = (low, high);
            }
        }

        let mut look = CarLook::default();
        let colour = |table: &Table, key: &str, slot: &mut Color| -> mlua::Result<()> {
            if let Some(rgb) = table.get::<Option<Table>>(key)? {
                *slot = Color::new(rgb.get(1)?, rgb.get(2)?, rgb.get(3)?, 1.0);
            }
            Ok(())
        };
        if let Some(car) = street.get::<Option<Table>>("car")? {
            if let Some(rows) = car.get::<Option<Vec<String>>>("pattern")? {
                let width = rows.first().map_or(0, |row| row.chars().count());
                if width == 0 || rows.iter().any(|row| row.chars().count() != width) {
                    return Err(fail("street.car.pattern needs equal, non-empty rows"));
                }
                look.pattern = rows;
            }
            if let Some(bodies) = car.get::<Option<Vec<Table>>>("bodies")? {
                if bodies.is_empty() {
                    return Err(fail("street.car.bodies needs a colour"));
                }
                look.bodies = bodies
                    .iter()
                    .map(|rgb| Ok(Color::new(rgb.get(1)?, rgb.get(2)?, rgb.get(3)?, 1.0)))
                    .collect::<mlua::Result<_>>()?;
            }
            colour(&car, "cabin", &mut look.cabin)?;
            colour(&car, "tail", &mut look.tail)?;
            colour(&car, "wheel", &mut look.wheel)?;
            colour(&car, "headlight", &mut look.headlight)?;
            if let Some(block) = car.get::<Option<f32>>("block")? {
                if !block.is_finite() || block <= 0.0 {
                    return Err(fail("street.car.block must be positive"));
                }
                look.block = block;
            }
        }
        if let Some(light) = street.get::<Option<Table>>("light")? {
            colour(&light, "beam", &mut look.beam)?;
            colour(&light, "glint", &mut look.glint)?;
            colour(&light, "wash", &mut look.wash)?;
            if let Some(strength) = light.get::<Option<f32>>("wash_strength")? {
                look.wash_strength = strength.clamp(0.0, 1.0);
            }
        }

        Ok(StreetView {
            glass,
            mirror: rect("mirror")?,
            floor: street
                .get::<Option<f32>>("floor")?
                .unwrap_or(glass.y - 50.0),
            timing,
            look,
        })
    }
}

/// A car in view: how far across the window it is and which way it is going.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Car {
    /// 0 when it enters the window, 1 when it has left.
    pub progress: f32,
    pub rightwards: bool,
    /// Picks one of the car colours.
    pub style: usize,
}

struct Passing {
    age: f32,
    duration: f32,
    rightwards: bool,
    style: usize,
}

pub struct Street {
    timing: Timing,
    random: u32,
    wait: f32,
    passing: Option<Passing>,
}

impl Default for Street {
    fn default() -> Self {
        Self::new(0x9e37_79b9)
    }
}

impl Street {
    pub fn new(seed: u32) -> Self {
        let mut street = Street {
            timing: Timing::default(),
            random: seed | 1,
            wait: 0.0,
            passing: None,
        };
        street.wait = street.between(street.timing.first);
        street
    }

    /// Starts the traffic over from a new seed, at the same pace.
    pub fn reseed(&mut self, seed: u32) {
        *self = Street {
            timing: self.timing,
            ..Street::new(seed)
        };
        self.wait = self.between(self.timing.first);
    }

    /// Changes how often cars come; the traffic starts over from its first gap if the pace is new.
    pub fn set_timing(&mut self, timing: Timing) {
        if timing != self.timing {
            self.timing = timing;
            self.passing = None;
            self.wait = self.between(timing.first);
        }
    }

    /// A number in 0..1 from a xorshift generator.
    fn next(&mut self) -> f32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 17;
        self.random ^= self.random << 5;
        (self.random >> 8) as f32 / (1u32 << 24) as f32
    }

    fn between(&mut self, (low, high): (f32, f32)) -> f32 {
        low + (high - low) * self.next()
    }

    pub fn advance(&mut self, delta_seconds: f32) {
        if let Some(passing) = &mut self.passing {
            passing.age += delta_seconds;
            if passing.age >= passing.duration {
                self.passing = None;
                self.wait = self.between(self.timing.gap);
            }
            return;
        }
        self.wait -= delta_seconds;
        if self.wait <= 0.0 {
            self.passing = Some(Passing {
                age: 0.0,
                duration: self.between(self.timing.crossing),
                rightwards: self.next() < 0.5,
                style: (self.next() * 3.0) as usize % 3,
            });
        }
    }

    pub fn car(&self) -> Option<Car> {
        self.passing.as_ref().map(|passing| Car {
            progress: (passing.age / passing.duration).clamp(0.0, 1.0),
            rightwards: passing.rightwards,
            style: passing.style,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_car_comes_crosses_and_goes() {
        let mut street = Street::new(7);
        assert!(street.car().is_none());
        let mut seen = false;
        let mut last = 0.0;
        for _ in 0..(60 * 60) {
            street.advance(1.0 / 60.0);
            if let Some(car) = street.car() {
                seen = true;
                assert!((0.0..=1.0).contains(&car.progress));
                last = car.progress;
            }
        }
        assert!(seen);
        assert!(last >= 0.0);
    }

    #[test]
    fn the_same_seed_gives_the_same_traffic() {
        let run = |seed| {
            let mut street = Street::new(seed);
            (0..3000)
                .map(|_| {
                    street.advance(0.1);
                    street.car()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(run(5), run(5));
        assert_ne!(run(5), run(6));
    }

    #[test]
    fn cars_are_spaced_out() {
        let mut street = Street::new(3);
        let mut starts = 0;
        let mut was = false;
        for _ in 0..(300 * 10) {
            street.advance(0.1);
            let now = street.car().is_some();
            starts += (now && !was) as u32;
            was = now;
        }
        // 300 s with a car at least every ~29 s and at most every ~11 s.
        assert!((8..=28).contains(&starts), "{starts}");
    }
}
