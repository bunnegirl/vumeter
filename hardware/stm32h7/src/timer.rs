pub use fugit::{ExtU32, TimerDurationU32, TimerInstantU32};
use rtic::rtic_monotonic::Monotonic;
use stm32h7xx_hal::{
    pac::{RCC, TIM2},
    rcc::CoreClocks,
};

#[no_mangle]
pub extern "Rust" fn external_now() -> TimerInstantU32<1_000_000> {
    crate::app::monotonics::now()
}

pub struct MonoTimer<T, const FREQ: u32>(T);

impl<const FREQ: u32> MonoTimer<TIM2, FREQ> {
    pub fn new(timer: TIM2, clocks: &CoreClocks) -> Self {
        let rcc = unsafe { &(*RCC::ptr()) };

        rcc.apb1lenr.modify(|_, w| w.tim2en().set_bit());
        rcc.apb1lrstr.modify(|_, w| w.tim2rst().set_bit());
        rcc.apb1lrstr.modify(|_, w| w.tim2rst().clear_bit());

        let pclk_mul = if clocks.ppre1() == 1 { 1 } else { 2 };
        let prescaler = clocks.pclk1().0 * pclk_mul / FREQ - 1;

        timer.psc.write(|w| w.psc().bits(prescaler as u16));
        timer.arr.write(|w| unsafe { w.bits(u32::MAX) });
        timer.egr.write(|w| w.ug().set_bit());
        timer.sr.modify(|_, w| w.uif().clear_bit());
        timer.cr1.modify(|_, w| w.cen().set_bit().udis().set_bit());

        MonoTimer(timer)
    }
}

impl<const FREQ: u32> Monotonic for MonoTimer<TIM2, FREQ> {
    type Instant = TimerInstantU32<FREQ>;
    type Duration = TimerDurationU32<FREQ>;

    unsafe fn reset(&mut self) {
        self.0.dier.modify(|_, w| w.cc1ie().set_bit());
    }

    #[inline(always)]
    fn now(&mut self) -> Self::Instant {
        Self::Instant::from_ticks(self.0.cnt.read().cnt().bits())
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        self.0
            .ccr1
            .write(|w| w.ccr().bits(instant.duration_since_epoch().ticks()));
    }

    fn clear_compare_flag(&mut self) {
        self.0.sr.modify(|_, w| w.cc1if().clear_bit());
    }

    #[inline(always)]
    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }
}
