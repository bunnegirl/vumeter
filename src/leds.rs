use core::convert::Infallible;
use stm32f4xx_hal::hal::digital::v2::OutputPin;

pub type LedLevel = Option<u8>;

pub struct LedPattern([bool; 6]);

impl From<[bool; 6]> for LedPattern {
    fn from(pattern: [bool; 6]) -> Self {
        Self(pattern)
    }
}

impl From<f32> for LedPattern {
    fn from(duty: f32) -> Self {
        Self::from(match duty {
            x if x > 99.0 => None,
            x if x > 66.0 => Some(5),
            x if x > 55.0 => Some(4),
            x if x > 44.0 => Some(3),
            x if x > 33.0 => Some(2),
            x if x > 22.0 => Some(1),
            x if x > 11.0 => Some(0),
            _ => None,
        })
    }
}

impl From<u8> for LedPattern {
    fn from(level: u8) -> Self {
        Self::from(Some(level))
    }
}

impl From<LedLevel> for LedPattern {
    fn from(level: LedLevel) -> LedPattern {
        let mut pattern = [false; 6];

        for index in 0..=5 {
            if let Some(level) = level {
                if level >= index {
                    pattern[index as usize] = true;
                }
            }
        }

        LedPattern(pattern)
    }
}

impl LedPattern {
    pub fn peak(mut self) -> Self {
        let mut found_peak = false;

        for index in (0..=5).rev() {
            if found_peak {
                self.0[index] = false;
            } else if self.0[index] == true {
                found_peak = true;
            }
        }

        self
    }
}

pub struct Leds<P1, P2, P3, P4, P5, P6>(pub P1, pub P2, pub P3, pub P4, pub P5, pub P6)
where
    P1: OutputPin<Error = Infallible>,
    P2: OutputPin<Error = Infallible>,
    P3: OutputPin<Error = Infallible>,
    P4: OutputPin<Error = Infallible>,
    P5: OutputPin<Error = Infallible>,
    P6: OutputPin<Error = Infallible>;

impl<P1, P2, P3, P4, P5, P6> Leds<P1, P2, P3, P4, P5, P6>
where
    P1: OutputPin<Error = Infallible>,
    P2: OutputPin<Error = Infallible>,
    P3: OutputPin<Error = Infallible>,
    P4: OutputPin<Error = Infallible>,
    P5: OutputPin<Error = Infallible>,
    P6: OutputPin<Error = Infallible>,
{
    pub fn get_led(&mut self, level: LedLevel) -> Option<&mut dyn OutputPin<Error = Infallible>> {
        match level {
            Some(0) => Some(&mut self.0),
            Some(1) => Some(&mut self.1),
            Some(2) => Some(&mut self.2),
            Some(3) => Some(&mut self.3),
            Some(4) => Some(&mut self.4),
            Some(5) => Some(&mut self.5),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        for index in 0..6 {
            if let Some(led) = self.get_led(Some(index)) {
                led.set_low().unwrap();
            }
        }
    }

    pub fn set(&mut self, pattern: LedPattern) {
        for (index, high) in pattern.0.iter().enumerate() {
            if let Some(led) = self.get_led(Some(index as u8)) {
                if *high {
                    led.set_high().unwrap();
                } else {
                    led.set_low().unwrap();
                }
            }
        }
    }
}
