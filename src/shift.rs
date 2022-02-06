use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use stm32f4xx_hal::{dwt::Delay, hal::digital::v2::OutputPin};

pub type ShiftRegister8 = dyn ShiftRegisterExt<8>;
pub type ShiftRegister24 = dyn ShiftRegisterExt<24>;

pub trait ShiftRegisterExt<const LEN: usize> {
    fn write(&mut self, data: usize);
}

impl<const LEN: usize, D, L, C> ShiftRegisterExt<LEN> for (Delay, D, L, C)
where
    D: OutputPin,
    L: OutputPin,
    C: OutputPin,
{
    fn write(&mut self, mut pattern: usize) {
        let (delay, data, latch, clock) = self;
        let mut index = 0;

        latch.set_low().ok();

        while index < LEN {
            if 1 & pattern > 0 {
                data.set_high().ok();
            }

            clock.set_high().ok();
            pattern >>= 1;

            delay.delay_us(1u32);

            clock.set_low().ok();
            data.set_low().ok();

            delay.delay_us(1u32);
            index += 1;
        }

        latch.set_high().ok();
    }
}
