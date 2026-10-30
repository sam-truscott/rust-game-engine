//! The street outside the pub window: cars that drive past at random intervals.

/// Seconds between one car leaving and the next appearing.
const GAP: (f32, f32) = (7.0, 25.0);
/// Seconds a car takes to cross the window.
const CROSSING: (f32, f32) = (2.4, 3.8);
/// The first car comes soon, so a new game shows one.
const FIRST_GAP: (f32, f32) = (3.0, 8.0);

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
            random: seed | 1,
            wait: 0.0,
            passing: None,
        };
        street.wait = street.between(FIRST_GAP);
        street
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
                self.wait = self.between(GAP);
            }
            return;
        }
        self.wait -= delta_seconds;
        if self.wait <= 0.0 {
            self.passing = Some(Passing {
                age: 0.0,
                duration: self.between(CROSSING),
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
