use crate::runtime::{State, State::*};
#[allow(unused_imports)]
use rtt_target::*;
use stm32f4xx_hal::{
    pac::TIM10,
    pwm::{PwmChannel, C1},
};

pub type BrightnessOutput = PwmChannel<TIM10, C1>;

#[derive(Debug, Clone, Copy)]
pub enum BrightnessLevel {
    High,
    Medium,
    Low,
}

impl Default for BrightnessLevel {
    fn default() -> Self {
        BrightnessLevel::High
    }
}

pub struct Brightness {
    output: BrightnessOutput,
}

impl Brightness {
    pub fn new(output: BrightnessOutput) -> Self {
        Self { output }
    }

    pub fn read(&mut self) {}

    pub fn write(&mut self, state: &State) {
        use BrightnessLevel::*;

        let max_duty = self.output.get_max_duty();

        match state {
            Running { brightness, .. } => {
                self.output.set_duty(
                    max_duty
                        / match brightness {
                            High => 1,
                            Medium => 2,
                            Low => 6,
                        },
                );
                self.output.enable();
            }
            _ => {
                self.output.disable();
            }
        };
    }

    pub fn clock(&mut self) {
        //
    }
}
