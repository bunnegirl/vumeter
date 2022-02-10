use crate::hardware::shift::*;
use crate::hardware::time;
use crate::hardware::TimeInstant;
use crate::runtime::{Message::*, State, State::*};
#[allow(unused_imports)]
use rtt_target::*;
use stm32f4xx_hal::gpio::*;

/// the number of rising and falling edges on the
/// clock pin that can occur per the left and right
/// input period
const CLOCKS_PER_INPUT: u32 = 96;
/// how many clocks to read before sending the read
/// average to state.
///
/// the lower this number the noisier the average,
/// the higher, the more delay you add.
///
/// using 96 * 16 results in a 30ms delay when the
/// clock is running at 24khz.
const CLOCKS_PER_READ: u32 = CLOCKS_PER_INPUT * 16;

trait MeterStateExt {
    fn levels(&self) -> (usize, usize);
}

impl MeterStateExt for &State {
    fn levels(&self) -> (usize, usize) {
        let mut left_result = 0;
        let mut right_result = 0;

        if let Running {
            left,
            right,
            peaks,
            levels,
            ..
        } = self
        {
            if *levels {
                left_result |= left.level;
                right_result |= right.level;
            }

            if *peaks {
                left_result |= left.peak;
                right_result |= right.peak;
            }
        }

        (left_result, right_result)
    }
}

pub type MeterRegister = ShiftRegister<
    24,
    (),
    Pin<Output<PushPull>, 'B', 5>,
    Pin<Output<PushPull>, 'B', 6>,
    Pin<Output<PushPull>, 'B', 7>,
>;

pub type MeterInputClock = Pin<Input<PullUp>, 'A', 8>;
pub type MeterInputLeft = Pin<Input<PullUp>, 'A', 10>;
pub type MeterInputRight = Pin<Input<PullUp>, 'A', 11>;

pub struct MeterInput {
    clock: MeterInputClock,
    left: MeterInputLeft,
    right: MeterInputRight,
    clock_count: u32,
    left_count: u32,
    right_count: u32,
}

impl MeterInput {
    pub fn new(clock: MeterInputClock, left: MeterInputLeft, right: MeterInputRight) -> Self {
        Self {
            clock_count: 0,
            left_count: 0,
            right_count: 0,
            clock,
            left,
            right,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeterChannel {
    pub level: usize,
    pub level_decay: TimeInstant,
    pub peak: usize,
    pub peak_decay: TimeInstant,
    pub calculated: usize,
}

impl Default for MeterChannel {
    fn default() -> Self {
        Self {
            calculated: 0,
            level: 0,
            level_decay: time::now(),
            peak: 0,
            peak_decay: time::now(),
        }
    }
}

pub struct Meter {
    input: MeterInput,
    pub register: MeterRegister,
}

impl Meter {
    pub fn new(input: MeterInput, register: MeterRegister) -> Self {
        Self { input, register }
    }

    pub fn read(&mut self) {
        let MeterInput {
            clock_count,
            clock,
            left,
            left_count,
            right,
            right_count,
        } = &mut self.input;

        clock.clear_interrupt_pending_bit();

        if *clock_count == CLOCKS_PER_READ {
            MeterUpdate(
                *left_count as f32 / CLOCKS_PER_READ as f32,
                *right_count as f32 / CLOCKS_PER_READ as f32,
            )
            .send();

            *clock_count = 0;
            *left_count = 0;
            *right_count = 0;
        }

        if left.is_high() {
            *left_count += 1;
        }

        if right.is_high() {
            *right_count += 1;
        }

        *clock_count += 1;
    }

    pub fn write(&mut self, state: &State) {
        let mut shift_data = 0;
        let (left, right) = state.levels();

        shift_data <<= 12;
        shift_data |= left.reverse_bits().rotate_left(12);

        shift_data <<= 12;
        shift_data |= right.reverse_bits().rotate_left(12);

        self.register.write((), shift_data);
    }

    pub fn clock(&mut self) {
        self.register.clock();
    }
}
