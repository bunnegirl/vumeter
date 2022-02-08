use heapless::Deque;
#[allow(unused_imports)]
use rtt_target::*;
use stm32f4xx_hal::hal::digital::v2::{OutputPin, PinState};

pub type ShiftBuffer<Id> = Deque<(Option<Id>, PinState, PinState, PinState), 2000>;

pub struct ShiftRegister<const LEN: usize, Id, Data, Latch, Clock> {
    pub buffer: ShiftBuffer<Id>,
    pub data: Data,
    pub latch: Latch,
    pub clock: Clock,
}

impl<const LEN: usize, Id, Data, Latch, Clock> ShiftRegister<LEN, Id, Data, Latch, Clock>
where
    Id: Copy,
    Data: OutputPin,
    Latch: OutputPin,
    Clock: OutputPin,
{
    pub fn clock(&mut self) -> Option<Id> {
        let Self {
            buffer,
            data,
            latch,
            clock,
        } = self;

        if let Some((id, data_state, latch_state, clock_state)) = buffer.pop_front() {
            data.set_state(data_state).ok();
            latch.set_state(latch_state).ok();
            clock.set_state(clock_state).ok();

            id
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn write(&mut self, id: Id, mut pattern: usize) {
        let Self { buffer, .. } = self;

        let mut index = 0;

        buffer
            .push_back((None, PinState::Low, PinState::Low, PinState::Low))
            .ok();

        while index < LEN {
            let data_state = if 1 & pattern > 0 && index <= LEN {
                PinState::High
            } else {
                PinState::Low
            };

            buffer
                .push_back((Some(id), data_state, PinState::Low, PinState::Low))
                .ok();

            buffer
                .push_back((None, data_state, PinState::Low, PinState::High))
                .ok();

            pattern >>= 1;
            index += 1;
        }

        buffer
            .push_back((None, PinState::Low, PinState::High, PinState::Low))
            .ok();

        buffer
            .push_back((None, PinState::Low, PinState::Low, PinState::Low))
            .ok();
    }
}
