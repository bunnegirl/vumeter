const METER_SIZE: usize = 9;

use stm32h7xx_hal::{
    gpio::{gpioa::*, gpioe::*, *},
    hal::digital::v2::*,
    Never,
};
use core::cmp::Ordering;
pub use Level::*;

pub const INACTIVE: Level = Level::Minus96;

pub enum LeftPin<MODE> {
    Clip(PA12<MODE>),
    Plus6(PA10<MODE>),
    Nominal(PA8<MODE>),
    Minus6(PA5<MODE>),
    Minus12(PA3<MODE>),
    Minus18(PA1<MODE>),
    Minus30(PA4<MODE>),
    Minus48(PA2<MODE>),
    Minus78(PA0<MODE>),
    Minus96,
}

impl LeftPin<Output<PushPull>> {
    fn pin(&mut self) -> Option<&mut dyn OutputPin<Error = Never>> {
        use LeftPin::*;
        
        match self {
            Clip(pin) => Some(pin),
            Plus6(pin) => Some(pin),
            Nominal(pin) => Some(pin),
            Minus6(pin) => Some(pin),
            Minus12(pin) => Some(pin),
            Minus18(pin) => Some(pin),
            Minus30(pin) => Some(pin),
            Minus48(pin) => Some(pin),
            Minus78(pin) => Some(pin),
            Minus96 => None,
        }
    }
}

pub enum RightPin<MODE> {
    Clip(PE2<MODE>),
    Plus6(PE4<MODE>),
    Nominal(PE6<MODE>),
    Minus6(PE11<MODE>),
    Minus12(PE13<MODE>),
    Minus18(PE15<MODE>),
    Minus30(PE10<MODE>),
    Minus48(PE12<MODE>),
    Minus78(PE14<MODE>),
    Minus96,
}

impl RightPin<Output<PushPull>> {
    fn pin(&mut self) -> Option<&mut dyn OutputPin<Error = Never>> {
        use RightPin::*;
        
        match self {
            Clip(pin) => Some(pin),
            Plus6(pin) => Some(pin),
            Nominal(pin) => Some(pin),
            Minus6(pin) => Some(pin),
            Minus12(pin) => Some(pin),
            Minus18(pin) => Some(pin),
            Minus30(pin) => Some(pin),
            Minus48(pin) => Some(pin),
            Minus78(pin) => Some(pin),
            Minus96 => None,
        }
    }
}

pub struct LevelPins<MODE>(pub Level, pub LeftPin<MODE>, pub RightPin<MODE>);

pub trait LevelActivityExt {
    fn is_active(&self) -> bool;
    fn is_inactive(&self) -> bool;
}

pub trait LevelToPatternExt {
    fn to_pattern(&self) -> Pattern;
}

pub trait LevelsToPatternsExt {
    fn to_patterns(&self) -> Patterns;
}

pub trait LevelPinsExt<MODE> {
    fn pins(self, left: LeftPin<MODE>, right: RightPin<MODE>) -> LevelPins<MODE>;
}

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Clip,
    Plus6,
    Nominal,
    Minus6,
    Minus12,
    Minus18,
    Minus30,
    Minus48,
    Minus78,
    Minus96,
}

impl LevelActivityExt for Level {
    fn is_active(&self) -> bool {
        !self.is_inactive()
    }

    fn is_inactive(&self) -> bool {
        self == &INACTIVE
    }
}

impl<MODE> LevelPinsExt<MODE> for Level {
    fn pins(self, left: LeftPin<MODE>, right: RightPin<MODE>) -> LevelPins<MODE> {
        LevelPins(self, left, right)
    }
}

impl LevelToPatternExt for Level {
    fn to_pattern(&self) -> Pattern {
        let index = (*self) as usize;
        let mut pattern = Pattern::new();

        pattern.set_up_to(index, true);

        pattern
    }
}

impl Into<&str> for Level {
    fn into(self) -> &'static str {
        match self {
            Clip => "clipping",
            Plus6 => "+6db",
            Nominal => "~0db",
            Minus6 => "-6db",
            Minus12 => "-12db",
            Minus18 => "-18db",
            Minus30 => "-24db",
            Minus48 => "-30db",
            Minus78 => "-36db",
            Minus96 => "-96db",
        }
    }
}

impl Into<f32> for Level {
    fn into(self) -> f32 {
        match self {
            Clip => 0.80833,
            Plus6 => 0.74167,
            Nominal => 0.68333,
            Minus6 => 0.61667,
            Minus12 => 0.55833,
            Minus18 => 0.50000,
            Minus30 => 0.37500,
            Minus48 => 0.19167,
            Minus78 => 0.01667,
            Minus96 => 0.00833,
        }
    }
}

impl Into<f32> for &Level {
    fn into(self) -> f32 {
        (*self).into()
    }
}

impl From<f32> for Level {
    fn from(level: f32) -> Self {
        match level {
            lvl if lvl > Clip.into() => Clip,
            lvl if lvl > Plus6.into() => Plus6,
            lvl if lvl > Nominal.into() => Nominal,
            lvl if lvl > Minus6.into() => Minus6,
            lvl if lvl > Minus12.into() => Minus12,
            lvl if lvl > Minus18.into() => Minus18,
            lvl if lvl > Minus30.into() => Minus30,
            lvl if lvl > Minus48.into() => Minus48,
            lvl if lvl > Minus78.into() => Minus78,
            _ => Minus96,
        }
    }
}

impl PartialEq<Level> for Level {
    fn eq(&self, rhs: &Level) -> bool {
        (*self as usize) == (*rhs as usize)
    }
}

impl PartialOrd<Level> for Level {
    fn partial_cmp(&self, rhs: &Level) -> Option<Ordering> {
        use Ordering::*;

        let lhs: f32 = self.into();
        let rhs: f32 = rhs.into();

        if lhs == rhs {
            Some(Equal)
        } else if lhs > rhs {
            Some(Greater)
        } else if lhs < rhs {
            Some(Less)
        } else {
            None
        }
    }
}

impl<MODE> PartialEq<LevelPins<MODE>> for Level {
    fn eq(&self, rhs: &LevelPins<MODE>) -> bool {
        let LevelPins(rhs, _, _) = rhs;

        self == rhs
    }
}

impl<MODE> PartialOrd<LevelPins<MODE>> for Level {
    fn partial_cmp(&self, rhs: &LevelPins<MODE>) -> Option<Ordering> {
        let LevelPins(rhs, _, _) = rhs;

        self.partial_cmp(rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Levels(pub Level, pub Level);

impl LevelActivityExt for Levels {
    fn is_active(&self) -> bool {
        self.0.is_active() || self.1.is_active()
    }

    fn is_inactive(&self) -> bool {
        !self.is_active()
    }
}

impl LevelsToPatternsExt for Levels {
    fn to_patterns(&self) -> Patterns {
        Patterns(self.0.to_pattern(), self.1.to_pattern())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Patterns(pub Pattern, pub Pattern);

pub trait PatternExt {
    fn to_array(self) -> [bool; METER_SIZE];
    fn is_bottom_high(&self) -> bool;
    fn is_top_high(&self) -> bool;
    fn is_high_at(&self, index: usize) -> bool;
    fn rotate_left(&mut self, amount: usize);
    fn rotate_right(&mut self, amount: usize);
    fn set_all(&mut self, value: bool);
    fn set_at(&mut self, at: usize, value: bool);
    fn set_down_to(&mut self, to: usize, value: bool);
    fn set_up_to(&mut self, to: usize, value: bool);
}

#[derive(Debug, Clone, Copy)]
pub struct Pattern {
    pattern: [bool; METER_SIZE],
}

impl Pattern {
    pub fn new() -> Self {
        Self {
            pattern: [false; METER_SIZE]
        }
    }
}

impl PatternExt for Pattern {
    fn to_array(self) -> [bool; METER_SIZE] {
        self.pattern
    }

    fn is_bottom_high(&self) -> bool {
        self.pattern[METER_SIZE - 1]
    }

    fn is_top_high(&self) -> bool {
        self.pattern[0]
    }

    fn is_high_at(&self, index: usize) -> bool {
        if index < METER_SIZE {
            self.pattern[index]
        } else {
            false
        }
    }

    fn rotate_left(&mut self, amount: usize) {
        self.pattern.rotate_left(amount);
    }

    fn rotate_right(&mut self, amount: usize) {
        self.pattern.rotate_right(amount);
    }

    fn set_all(&mut self, value: bool) {
        self.pattern = [value; METER_SIZE];
    }

    fn set_at(&mut self, at: usize, value: bool) {
        if at < METER_SIZE {
            self.pattern[at] = value
        }
    }

    fn set_down_to(&mut self, to: usize, value: bool) {
        if to < METER_SIZE {
            for old in self.pattern[0..=to].iter_mut() {
                *old = value
            }
        }
    }

    fn set_up_to(&mut self, to: usize, value: bool) {
        if to < METER_SIZE {
            for old in self.pattern[to..].iter_mut() {
                *old = value;
            }
        }
    }
}

pub type Meter<MODE> = [LevelPins<MODE>; METER_SIZE];

pub trait MeterExt {
    fn clear(&mut self);
    fn set(&mut self, left: Pattern, right: Pattern);
}

impl MeterExt for Meter<Output<PushPull>> {
    fn clear(&mut self) {}

    fn set(&mut self, left: Pattern, right: Pattern) {
        let left = left.to_array();
        let right = right.to_array();

        for pins in self {
            let LevelPins(pin_level, left_pin, right_pin) = pins;
            let index = (*pin_level) as usize;

            if let Some(left_pin) = left_pin.pin() {
                if left[index] {
                    left_pin.set_high().ok();
                } else {
                    left_pin.set_low().ok();
                }
            }

            if let Some(right_pin) = right_pin.pin() {
                if right[index] {
                    right_pin.set_high().ok();
                } else {
                    right_pin.set_low().ok();
                }
            }
        }
    }
}
