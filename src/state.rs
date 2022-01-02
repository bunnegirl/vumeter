use crate::bus::*;
use crate::meter::*;
use crate::mcu::monotonics;
use fugit::{ExtU32, Instant};

pub use Device::*;
pub use Power::*;
pub use Signal::*;
pub use Timeout::*;

#[derive(Debug, Clone, Copy)]
pub enum Device {
    Headphones,
    Speakers,
}

#[derive(Debug, Clone, Copy)]
pub enum Power {
    Booting,
    PowerOff,
    PowerOn,
}

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Muted,
    Unmuted,
}

#[derive(Debug, Clone, Copy)]
pub enum Timeout {
    Running(Instant<u32, 1, 1000000_u32>),
    Idling(Pattern),
}

fn timeout() -> Instant<u32, 1, 1000000_u32> {
    monotonics::now() + 30.secs()
}

#[derive(Debug, Clone, Copy)]
pub struct State(Power, Signal, Device, Timeout);

impl State {
    pub fn new() -> Self {
        State(Booting, Unmuted, Headphones, Running(timeout()))
    }

    pub fn with_device(self, device: Device) -> Self {
        State(self.0, self.1, device, self.3)
    }

    pub fn with_power(self, power: Power) -> Self {
        State(power, self.1, self.2, self.3)
    }

    pub fn with_signal(self, signal: Signal) -> Self {
        State(self.0, signal, self.2, self.3)
    }

    pub fn with_timeout(self, timeout: Timeout) -> Self {
        State(self.0, self.1, self.2, timeout)
    }
}

pub fn modify_state(state: State, msg: StateMsg) -> State {
    match (state, msg) {
        // set initial state
        (State(Booting, ..), Initialise) => {
            ToMcu(SetPower(PowerOn)).send();
            ToMcu(SetDevice(Headphones)).send();
            ToMcu(SetMute(Unmuted)).send();

            State(PowerOn, Unmuted, Headphones, Running(timeout()))
        }

        // power on dsp
        (State(PowerOff, ..), TogglePower) => {
            ToMcu(SetPower(PowerOn)).send();

            state.with_power(PowerOn)
        }

        // power off dsp
        (State(PowerOn, ..), TogglePower) => {
            ToMcu(SetPower(PowerOff)).send();

            state.with_power(PowerOff)
        }

        // switch to headphones
        (State(PowerOn, _, Speakers, _), ToggleDevice) => {
            ToMcu(SetDevice(Headphones)).send();

            state.with_device(Headphones)
        }

        // switch to speakers
        (State(PowerOn, _, Headphones, _), ToggleDevice) => {
            ToMcu(SetDevice(Speakers)).send();

            state.with_device(Speakers)
        }

        // mute output
        (State(PowerOn, Unmuted, ..), ToggleMute) => {
            ToMcu(SetMute(Muted)).send();

            state.with_signal(Muted)
        }

        // unmute output
        (State(PowerOn, Muted, ..), ToggleMute) => {
            ToMcu(SetMute(Unmuted)).send();

            state.with_signal(Unmuted)
        }

        // update meter display
        (State(PowerOn, Unmuted, _, Running(time)), UpdateMeter(levels)) => {
            let patterns = levels.to_patterns();

            ToMcu(SetMeter(patterns)).send();

            state.with_timeout(Running(if levels.is_active() { timeout() } else { time }))
        }

        // timeout
        (State(PowerOn, Unmuted, _, Running(time)), Clock(_)) => {
            if time < monotonics::now() {
                let mut pattern = Pattern::new();

                pattern.set_at(0, true);
                pattern.rotate_left(1);

                state.with_timeout(Idling(pattern))
            } else {
                state.with_timeout(Running(time))
            }
        }

        // idling
        (State(PowerOn, Unmuted, _, Idling(mut pattern)), Clock(count)) => {
            if count % 50 == 0 {
                pattern.rotate_left(1);

                ToMcu(SetMeter(Patterns(pattern, pattern))).send();
            }

            state.with_timeout(Idling(pattern))
        }

        // resume 
        (State(PowerOn, Unmuted, _, Idling(_)), UpdateMeter(levels)) => {
            if levels.is_active() {
                let patterns = levels.to_patterns();

                ToMcu(SetMeter(patterns)).send();

                state.with_timeout(Running(timeout()))
            } else {
                state
            }
        }

        (state, _) => state,
    }
}
