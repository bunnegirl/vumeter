const METER_SIZE: usize = 9;

use stm32h7xx_hal::{
    gpio::{gpioa::*, gpioe::*, *},
    hal::digital::v2::*,
    Never,
};
use core::cmp::Ordering;
pub use Level::*;

pub enum LeftPin<MODE> {
    Clip(PA12<MODE>),
    Plus6(PA10<MODE>),
    Nominal(PA8<MODE>),
    Minus6(PA5<MODE>),
    Minus12(PA3<MODE>),
    Minus18(PA1<MODE>),
    Minus24(PA4<MODE>),
    Minus30(PA2<MODE>),
    Minus36(PA0<MODE>),
    Detect,
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
            Minus24(pin) => Some(pin),
            Minus30(pin) => Some(pin),
            Minus36(pin) => Some(pin),
            _ => None,
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
    Minus24(PE10<MODE>),
    Minus30(PE12<MODE>),
    Minus36(PE14<MODE>),
    Detect,
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
            Minus24(pin) => Some(pin),
            Minus30(pin) => Some(pin),
            Minus36(pin) => Some(pin),
            _ => None,
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
    Minus24,
    Minus30,
    Minus36,
    Detect,
}

impl LevelActivityExt for Level {
    fn is_active(&self) -> bool {
        self > &Level::Detect
    }

    fn is_inactive(&self) -> bool {
        self <= &Level::Detect
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
        let mut pattern = Pattern::default();

        pattern.set_up_to(index, true);

        pattern
    }
}

impl From<Level> for &str {
    fn from(level: Level) -> &'static str {
        match level {
            Clip => "clipping",
            Plus6 => "+6db",
            Nominal => "~0db",
            Minus6 => "-3db",
            Minus12 => "-6db",
            Minus18 => "-9db",
            Minus24 => "-12db",
            Minus30 => "-15db",
            Minus36 => "-18db",
            Detect => "detected",
        }
    } 
}

impl From<Level> for f32 {
    fn from(level: Level) -> f32 {
        match level {
            Clip => 0.9750,
            Plus6 => 0.8650,
            Nominal => 0.8150,
            Minus6 => 0.7750,
            Minus12 => 0.7350,
            Minus18 => 0.6950,
            Minus24 => 0.6450,
            Minus30 => 0.6050,
            Minus36 => 0.5650,
            Detect => 0.3950,
        }
    } 
}

impl From<&Level> for f32 {
    fn from(level: &Level) -> f32 {
        (*level).into()
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
            lvl if lvl > Minus24.into() => Minus24,
            lvl if lvl > Minus30.into() => Minus30,
            lvl if lvl > Minus36.into() => Minus36,
            _ => Detect,
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

impl Default for Pattern {
    fn default() -> Self {
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

            if index < left.len() {
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
}
