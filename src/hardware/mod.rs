pub mod brightness;
pub mod control;
pub mod debounce;
pub mod keypad;
pub mod meter;
pub mod monotonic;
pub mod shift;

pub use crate::hardware::inner::monotonics as time;
pub use crate::hardware::inner::TimerInstant;

use crate::hardware::brightness::*;
use crate::hardware::control::*;
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
        brightness: Brightness,
        control: Control,
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

        let audio_output_dsp = gpiob.pb12.into_push_pull_output();
        let audio_output_ctrl = gpiob.pb13.into_push_pull_output();
        let audio_mute_ctrl = gpiob.pb14.into_push_pull_output();

        let brightness_output =
            Timer::new(cx.device.TIM10, &clocks).pwm(gpiob.pb8.into_alternate(), 24.khz());

        let mut meter_clock = gpioa.pa8.into_pull_up_input();

        meter_clock.make_interrupt_source(&mut syscfg);
        meter_clock.enable_interrupt(&mut cx.device.EXTI);
        meter_clock.trigger_on_edge(&mut cx.device.EXTI, Edge::RisingFalling);

        let meter_input = MeterInput::new(
            meter_clock,
            gpioa.pa10.into_pull_up_input(),
            gpioa.pa11.into_pull_up_input(),
        );

        let meter_register = MeterRegister {
            buffer: ShiftBuffer::new(),
            data: gpiob.pb5.into_push_pull_output(),
            latch: gpiob.pb6.into_push_pull_output(),
            clock: gpiob.pb7.into_push_pull_output(),
        };

        let key_trigger = gpiob.pb4.into_pull_down_input();
        let key_register = KeyRegister {
            buffer: ShiftBuffer::new(),
            data: gpiob.pb3.into_push_pull_output(),
            latch: gpioa.pa15.into_push_pull_output(),
            clock: gpioa.pa12.into_push_pull_output(),
        };

        keypad::spawn().ok();
        clock::spawn().ok();

        Booted.send();

        (
            Shared {
                brightness: Brightness::new(brightness_output),
                control: Control::new(audio_output_dsp, audio_output_ctrl, audio_mute_ctrl),
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
            control,
            brightness,
            meter,
            state,
        ]
    )]
    fn idle(cx: idle::Context) -> ! {
        let idle::SharedResources {
            mut control,
            mut brightness,
            mut meter,
            mut state,
        } = cx.shared;

        loop {
            if let Some(msg) = Q.dequeue() {
                state.lock(|state| {
                    *state = state.recv(msg);

                    brightness.lock(|brightness| brightness.write(state));
                    control.lock(|control| control.write(state));
                    meter.lock(|meter| meter.write(state));
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
        binds = EXTI9_5,
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
