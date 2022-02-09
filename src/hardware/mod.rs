pub mod debounce;
pub mod keypad;
pub mod meter;
pub mod monotonic;
pub mod shift;

pub use crate::hardware::inner::monotonics as time;
pub use crate::hardware::inner::TimerInstant;

use crate::hardware::keypad::*;
use crate::hardware::meter::*;
use crate::hardware::monotonic::*;
use crate::hardware::shift::*;
use crate::runtime::{Message::*, State, Q};
use fugit::{ExtU32, Instant};
use rtt_target::*;
use stm32f4xx_hal::{gpio::*, pac, prelude::*, timer::Timer};

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [SPI1, SPI2, SPI3])]
mod inner {
    use super::*;

    #[monotonic(binds = TIM2, default = true)]
    type MonotonicTimer = MonoTimer<pac::TIM2, 8_000_000>;
    pub type TimerInstant = Instant<u32, 1, 8_000_000>;

    #[shared]
    struct Shared {
        keypad: Keypad,
        meter: Meter,
        state: State,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let mut syscfg = cx.device.SYSCFG.constrain();
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.freeze();
        let mono = MonoTimer::new(cx.device.TIM2, &clocks);

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();

        let mut brightness =
            Timer::new(cx.device.TIM10, &clocks).pwm(gpiob.pb8.into_alternate(), 48.khz());
        let max_duty = brightness.get_max_duty();

        brightness.set_duty(max_duty);
        brightness.enable();

        let meter_input = MeterInput::new(
            Timer::new(cx.device.TIM1, &clocks).pwm_input(24.khz(), gpioa.pa8.into_alternate()),
            gpioa.pa10.into_pull_up_input(),
            gpioa.pa11.into_pull_up_input(),
        );

        let meter_register = MeterRegister {
            buffer: ShiftBuffer::new(),
            data: gpiob.pb5.into_push_pull_output(),
            latch: gpiob.pb6.into_push_pull_output(),
            clock: gpiob.pb7.into_push_pull_output(),
        };

        let mut key_trigger = gpiob.pb4.into_pull_down_input();
        let key_register = KeyRegister {
            buffer: ShiftBuffer::new(),
            data: gpiob.pb3.into_push_pull_output(),
            latch: gpioa.pa15.into_push_pull_output(),
            clock: gpioa.pa12.into_push_pull_output(),
        };

        key_trigger.make_interrupt_source(&mut syscfg);
        key_trigger.enable_interrupt(&mut cx.device.EXTI);
        key_trigger.trigger_on_edge(&mut cx.device.EXTI, Edge::Rising);

        keypad::spawn().ok();
        clock::spawn().ok();

        Booted.send();

        (
            Shared {
                keypad: Keypad::new(key_trigger, key_register),
                meter: Meter::new(meter_input, meter_register),
                state: State::Booting,
            },
            Local {},
            init::Monotonics(mono),
        )
    }

    #[idle(
        shared = [
            keypad,
            meter,
            state,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
        let idle::SharedResources {
            keypad: _,
            mut meter,
            mut state,
        } = cx.shared;

        loop {
            if let Some(msg) = Q.dequeue() {
                state.lock(|state| {
                    *state = state.recv(msg);

                    meter.lock(|meter| meter.write(state));
                    // keypad.lock(|keypad| keypad.write());
                });
            }
        }
    }

    #[task(
        priority = 1,
        shared = [
            keypad,
            meter,
        ],
    )]
    fn clock(cx: clock::Context) {
        let clock::SharedResources {
            mut keypad,
            mut meter,
        } = cx.shared;

        meter.lock(|meter| {
            meter.clock();
        });

        keypad.lock(|keypad| {
            keypad.clock();
        });

        clock::spawn_after(50.micros()).ok();
    }

    #[task(
        priority = 2,
        shared = [
            keypad,
        ],
    )]
    fn keypad(cx: keypad::Context) {
        let keypad::SharedResources { mut keypad } = cx.shared;

        keypad.lock(|keypad| keypad.read());

        keypad::spawn_after(50.millis()).ok();
    }

    #[task(
        binds = TIM1_CC,
        priority = 2,
        shared = [
            meter,
        ]
    )]
    fn meter(cx: meter::Context) {
        let meter::SharedResources { mut meter } = cx.shared;

        meter.lock(|meter| meter.read());
    }
}
