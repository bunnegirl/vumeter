use stm32f4xx_hal::
    hal::digital::v2::OutputPin;
use core::convert::Infallible;
use rtt_target::{rprintln};

pub type LedLevel = Option<u8>;
pub type LedsPattern = [bool; 6];

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

    pub fn set_level(&mut self, level: LedLevel) {
        for index in 0..6 {
            let led = self.get_led(Some(index));

            if let (Some(level), Some(led)) = (level, led) {
                if level >= index {
                    led.set_high().unwrap();
                } else {
                    led.set_low().unwrap();
                }
            }
        }
    }

    pub fn set_pattern(&mut self, pattern: LedsPattern) {
        for (index, high) in pattern.iter().enumerate() {
            if let Some(led) = self.get_led(Some(index as u8)) {
                if *high {
                    led.set_high().unwrap();
                } else {
                    led.set_low().unwrap();
                }
            }
        }
    }

    pub fn reset_pattern(&mut self, pattern: LedsPattern) {
        for (index, high) in pattern.iter().enumerate() {
            if *high {
                if let Some(led) = self.get_led(Some(index as u8)) {
                    led.set_low().unwrap();
                }
            }
        }
    }
}