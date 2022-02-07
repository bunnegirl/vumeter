use crate::debounce::*;
use crate::shift::*;
use crate::state::*;
use stm32f4xx_hal::{dwt::Delay, gpio::*};

pub type KeyTrigger = Pin<Input<PullDown>, 'B', 4>;

pub type KeyRegister = (
    Delay,
    Pin<Output<PushPull>, 'B', 3>,
    Pin<Output<PushPull>, 'A', 15>,
    Pin<Output<PushPull>, 'A', 12>,
);

pub struct Keypad {
    trigger: KeyTrigger,
    register: KeyRegister,
}

impl Keypad {
    pub fn new(trigger: KeyTrigger, register: KeyRegister) -> Self {
        Self { trigger, register }
    }

    pub fn write_output(&mut self, mut _state: &mut State) {
        let register = &mut self.register as &mut ShiftRegister8;

        register.write(0b1111_1111);
    }

    pub fn read_input(&mut self, mut _state: &mut State, debouncer: &mut Debouncer<8>) {
        let trigger = &mut self.trigger;
        let register = &mut self.register as &mut ShiftRegister8;

        trigger.clear_interrupt_pending_bit();

        for index in 0..8 {
            register.write(1 << index);

            if trigger.is_high() {
                if debouncer.is_ok(1 + index) {
                    KeypadUpdate(index).send();
                }

                debouncer.update(1 + index, 50);
            }
        }

        register.write(0b1111_1111);
    }
}
