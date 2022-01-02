use crate::state::{Device, Power, Signal};
use crate::meter::{Levels, Patterns};
use heapless::mpmc::Q8;

pub use Busses::*;
pub use McuMsg::*;
pub use StateMsg::*;

pub static Q: Q8<Busses> = Q8::new();

pub enum Busses {
    ToMcu(McuMsg),
    ToState(StateMsg),
}

pub enum McuMsg {
    SetDevice(Device),
    SetMute(Signal),
    SetPower(Power),
    SetMeter(Patterns),
}

impl McuMsg {
    pub fn send(self) {
        Q.enqueue(ToMcu(self)).ok();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StateMsg {
    Clock(u32),
    Initialise,
    ToggleDevice,
    ToggleMute,
    TogglePower,
    UpdateMeter(Levels),
}

impl StateMsg {
    pub fn send(self) {
        Q.enqueue(ToState(self)).ok();
    }
}