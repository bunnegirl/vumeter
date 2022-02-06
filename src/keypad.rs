use crate::debounce::*;
use crate::shift::*;
use crate::state::*;
use stm32f4xx_hal::{dwt::Delay, gpio::*, prelude::*};

trait KeypadStateExt {
    fn indicators(&mut self) -> usize;
}

impl KeypadStateExt for &mut State {
    fn indicators(&mut self) -> usize {
        let mut result = 0;

        if let VolumeMeter(_, _, peaks, levels, headphones, speakers) = self {
            if let SpeakerOutput::On = speakers {
                result += 1;
            }

            result <<= 1;

            if let HeadphoneOutput::On = headphones {
                result += 1;
            }

            result <<= 1;

            if let ShowLevels::On = levels {
                result += 1;
            }

            result <<= 1;

            if let ShowPeaks::On = peaks {
                result += 1;
            }
        }

        result
    }
}

pub type KeyTrigger = Pin<Input<PullDown>, 'B', 4>;

pub type KeyRegister = (
    Delay,
    Pin<Output<PushPull>, 'B', 3>,
    Pin<Output<PushPull>, 'A', 15>,
    Pin<Output<PushPull>, 'A', 12>,
);

pub struct Keypad {
    key_delay: Delay,
    key_trigger: KeyTrigger,
    key_register: KeyRegister,
}

impl Keypad {
    pub fn new(key_delay: Delay, key_trigger: KeyTrigger, key_register: KeyRegister) -> Self {
        Self {
            key_delay,
            key_trigger,
            key_register,
        }
    }

    pub fn write_output(&mut self, mut state: &mut State) {
        let key_register = &mut self.key_register as &mut ShiftRegister8;

        key_register.write(0b1111_0000 + state.indicators());
    }

    pub fn read_input(&mut self, mut state: &mut State, debouncer: &mut Debouncer<5>) {
        let key_delay = &mut self.key_delay;
        let key_trigger = &mut self.key_trigger;
        let key_register = &mut self.key_register as &mut ShiftRegister8;
        let indicators = state.indicators();

        key_trigger.clear_interrupt_pending_bit();

        if debouncer.is_ok(0) {
            key_delay.delay_ms(50u32);

            while key_trigger.is_high() {
                for index in 0..5 {
                    key_register.write((1 << index << 4) + indicators);
                    key_delay.delay_ms(1u32);

                    if key_trigger.is_high() {
                        if debouncer.is_ok(1 + index) {
                            KeypadUpdate(index).send();
                        }

                        debouncer.update(1 + index, 50);
                    }
                }

                key_register.write(0b1111_0000);
                key_delay.delay_ms(1u32);
            }
        }

        debouncer.update(0, 50);
    }
}
