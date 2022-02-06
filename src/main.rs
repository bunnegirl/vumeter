#![no_main]
#![no_std]

mod debounce;
mod keypad;
mod meter;
mod monotonic;
mod shift;
mod state;

use crate::debounce::*;
use crate::keypad::*;
use crate::meter::*;
use crate::monotonic::MonoTimer;
use crate::state::*;
use core::panic::PanicInfo;
use cortex_m::asm::nop;
use fugit::{ExtU32, Instant};
use rtt_target::*;
use stm32f4xx_hal::{dwt::DwtExt, gpio::*, pac, prelude::*, timer::Timer};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        nop();
    }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [SPI1, SPI3])]
mod app {
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
        let dwt = cx.core.DWT.constrain(cx.core.DCB, &clocks);

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();

        let meter_input = MeterInput::new(
            Timer::new(cx.device.TIM1, &clocks).pwm_input(24.khz(), gpioa.pa8.into_alternate()),
            gpioa.pa10.into_pull_up_input(),
            gpioa.pa11.into_pull_up_input(),
        );

        let meter_register = (
            dwt.delay(),
            gpiob.pb5.into_push_pull_output(),
            gpiob.pb6.into_push_pull_output(),
            gpiob.pb7.into_push_pull_output(),
        );

        let key_delay = dwt.delay();
        let mut key_trigger = gpiob.pb4.into_pull_down_input();
        let key_register = (
            dwt.delay(),
            gpiob.pb3.into_push_pull_output(),
            gpioa.pa15.into_push_pull_output(),
            gpioa.pa12.into_push_pull_output(),
        );

        key_trigger.make_interrupt_source(&mut syscfg);
        key_trigger.enable_interrupt(&mut cx.device.EXTI);
        key_trigger.trigger_on_edge(&mut cx.device.EXTI, Edge::Rising);

        timer::spawn().ok();

        Initialise.send();

        (
            Shared {
                keypad: Keypad::new(key_delay, key_trigger, key_register),
                meter: Meter::new(meter_input, meter_register),
                state: State::Uninitialised,
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
            mut keypad,
            mut meter,
            mut state,
        } = cx.shared;

        loop {
            if let Some(msg) = Q.dequeue() {
                state.lock(|state| {
                    *state = state.recv(msg);

                    meter.lock(|meter| meter.write_output(state));
                    keypad.lock(|keypad| keypad.write_output(state));
                });
            }
        }
    }

    #[task()]
    fn timer(_cx: timer::Context) {
        MeterDecay.send();

        timer::spawn_after(50.millis()).ok();
    }

    #[task(
        binds = EXTI4,
        shared = [
            keypad,
            state,
        ],
        local = [
            debouncer: Debouncer<5> = Debouncer::new(),
        ],
    )]
    fn keypad_read(cx: keypad_read::Context) {
        let keypad_read::SharedResources {
            mut keypad,
            mut state,
        } = cx.shared;
        let keypad_read::LocalResources { debouncer } = cx.local;

        state.lock(|state| {
            keypad.lock(|keypad| keypad.read_input(state, debouncer));
        });
    }

    #[task(
        binds = TIM1_CC,
        shared = [
            meter,
            state
        ]
    )]
    fn meter_read(cx: meter_read::Context) {
        let meter_read::SharedResources {
            mut meter,
            mut state,
        } = cx.shared;

        state.lock(|state| {
            meter.lock(|meter| meter.read_input(state));
        });
    }
}
