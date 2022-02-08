use crate::debounce::*;
use crate::shift::*;
use crate::state::*;
#[allow(unused_imports)]
use rtt_target::*;
use stm32f4xx_hal::gpio::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Unassigned,
    TogglePeaks,
    ToggleLevels,
    ToggleHeadphones,
    ToggleSpeakers,
}

pub type KeyTrigger = Pin<Input<PullDown>, 'B', 4>;

pub type KeyRegister = ShiftRegister<
    8,
    Key,
    Pin<Output<PushPull>, 'B', 3>,
    Pin<Output<PushPull>, 'A', 15>,
    Pin<Output<PushPull>, 'A', 12>,
>;

pub struct Keypad {
    debouncer: Debouncer<8, Key>,
    trigger: KeyTrigger,
    register: KeyRegister,
}

impl Keypad {
    pub fn new(trigger: KeyTrigger, register: KeyRegister) -> Self {
        Self {
            debouncer: Debouncer::new(),
            trigger,
            register,
        }
    }

    pub fn read(&mut self) {
        use Key::*;

        let trigger = &mut self.trigger;

        trigger.clear_interrupt_pending_bit();

        self.register.write(ToggleSpeakers, 0b1000_0000);
        self.register.write(Unassigned, 0b0100_0000);
        self.register.write(TogglePeaks, 0b0010_0000);
        self.register.write(Unassigned, 0b0001_0000);
        self.register.write(ToggleLevels, 0b0000_1000);
        self.register.write(Unassigned, 0b0000_0100);
        self.register.write(ToggleHeadphones, 0b0000_0010);
        self.register.write(Unassigned, 0b0000_0001);
    }

    pub fn write(&mut self) {}

    pub fn clock(&mut self) {
        let trigger = &mut self.trigger;

        if let Some(id) = self.register.clock() {
            if trigger.is_high() {
                if self.debouncer.is_ok(id) {
                    // rprintln!("-{:?}-", );
                    KeypadUpdate(id).send();
                }

                self.debouncer.update(id, 50);
            }
        }
    }
}
