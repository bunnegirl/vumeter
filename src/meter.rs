use crate::app::monotonics as time;
use crate::app::TimerInstant;
use crate::shift::*;
use crate::state::*;
use stm32f4xx_hal::{
    dwt::Delay,
    gpio::{gpioa::*, *},
    pac::TIM1,
    pwm_input::PwmInput,
};

trait MeterStateExt {
    fn levels(&mut self) -> (usize, usize);
}

impl MeterStateExt for &mut State {
    fn levels(&mut self) -> (usize, usize) {
        let mut left_result = 0;
        let mut right_result = 0;

        if let VolumeMeter {
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

pub type MeterRegister = (
    Delay,
    Pin<Output<PushPull>, 'B', 5>,
    Pin<Output<PushPull>, 'B', 6>,
    Pin<Output<PushPull>, 'B', 7>,
);

pub type MeterInputClock = PwmInput<TIM1, PA8<Alternate<PushPull, 1>>>;
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
    pub level_decay: TimerInstant,
    pub peak: usize,
    pub peak_decay: TimerInstant,
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
    register: MeterRegister,
}

impl Meter {
    pub fn new(input: MeterInput, register: MeterRegister) -> Self {
        Self { input, register }
    }

    pub fn read_input(&mut self, mut _state: &mut State) {
        let MeterInput {
            clock_count,
            clock,
            left,
            left_count,
            right,
            right_count,
        } = &mut self.input;

        if clock.is_valid_capture() {
            if *clock_count == 220 {
                MeterUpdate(*left_count as f32 / 220.0, *right_count as f32 / 220.0).send();

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
    }

    pub fn write_output(&mut self, mut state: &mut State) {
        let meter_register = &mut self.register as &mut ShiftRegister24;

        let mut shift_data = 0;
        let (left, right) = state.levels();

        shift_data <<= 12;
        shift_data |= left.reverse_bits().rotate_left(12);

        shift_data <<= 12;
        shift_data |= right.reverse_bits().rotate_left(12);

        meter_register.write(shift_data);
    }
}
