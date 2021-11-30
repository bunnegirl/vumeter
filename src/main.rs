#![no_main]
#![no_std]
#![feature(trait_alias)]

mod mono_timer;

use core::panic::PanicInfo;
use crate::mono_timer::MonoTimer;
use core::convert::Infallible;
use fugit::ExtU32;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    gpio::{gpioa::*, gpiob::*, gpioc::*, *},
    hal::digital::v2::OutputPin,
    pac,
    prelude::*,
    pwm_input::PwmInput,
    stm32::{TIM1, TIM2, TIM3, TIM4},
    timer::Timer as HalTimer,
};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        cortex_m::asm::nop();
    }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI4])]
mod app {
    use super::*;

    const REFRESH: u32 = 8333;

    #[monotonic(binds = TIM5, default = true)]
    type Timer = MonoTimer<pac::TIM5, 8_000_000>;

    type Led = dyn OutputPin<Error = Infallible>;

    #[shared]
    struct Shared {
        headphones_left: u8,
        headphones_right: u8,
        speakers_left: u8,
        speakers_right: u8,
    }

    #[local]
    struct Local {
        headphones_left_pwm: PwmInput<TIM1, PA8<Alternate<1>>>,
        headphones_right_pwm: PwmInput<TIM2, PA15<Alternate<1>>>,
        speakers_left_pwm: PwmInput<TIM3, PB4<Alternate<2>>>,
        speakers_right_pwm: PwmInput<TIM4, PB6<Alternate<2>>>,
        headphones_leds_0_pin: (PC13<Output<PushPull>>, PA3<Output<PushPull>>),
        headphones_leds_1_pin: (PC14<Output<PushPull>>, PA4<Output<PushPull>>),
        headphones_leds_2_pin: (PC15<Output<PushPull>>, PA5<Output<PushPull>>),
        headphones_leds_3_pin: (PA0<Output<PushPull>>, PA6<Output<PushPull>>),
        headphones_leds_4_pin: (PA1<Output<PushPull>>, PA7<Output<PushPull>>),
        headphones_leds_5_pin: (PA2<Output<PushPull>>, PB0<Output<PushPull>>),
        speakers_leds_0_pin: (PB7<Output<PushPull>>, PA11<Output<PushPull>>),
        speakers_leds_1_pin: (PB5<Output<PushPull>>, PA10<Output<PushPull>>),
        speakers_leds_2_pin: (PB3<Output<PushPull>>, PB15<Output<PushPull>>),
        speakers_leds_3_pin: (PB10<Output<PushPull>>, PB14<Output<PushPull>>),
        speakers_leds_4_pin: (PB2<Output<PushPull>>, PB13<Output<PushPull>>),
        speakers_leds_5_pin: (PB1<Output<PushPull>>, PB12<Output<PushPull>>),
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let rcc = cx.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(25.mhz())
            .require_pll48clk()
            .sysclk(84.mhz())
            .hclk(84.mhz())
            .pclk1(42.mhz())
            .pclk2(84.mhz())
            .freeze();

        let mono = Timer::new(cx.device.TIM5, &clocks);

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();
        let gpioc = cx.device.GPIOC.split();

        let headphones_left_pwm = HalTimer::new(cx.device.TIM1, &clocks)
            .pwm_input(48u32.khz(), gpioa.pa8.into_alternate());
        let headphones_right_pwm = HalTimer::new(cx.device.TIM2, &clocks)
            .pwm_input(48u32.khz(), gpioa.pa15.into_alternate());

        let speakers_left_pwm = HalTimer::new(cx.device.TIM3, &clocks)
            .pwm_input(48u32.khz(), gpiob.pb4.into_alternate());
        let speakers_right_pwm = HalTimer::new(cx.device.TIM4, &clocks)
            .pwm_input(48u32.khz(), gpiob.pb6.into_alternate());

        let headphones_left_0_pin = gpioc.pc13.into_push_pull_output();
        let headphones_left_1_pin = gpioc.pc14.into_push_pull_output();
        let headphones_left_2_pin = gpioc.pc15.into_push_pull_output();
        let headphones_left_3_pin = gpioa.pa0.into_push_pull_output();
        let headphones_left_4_pin = gpioa.pa1.into_push_pull_output();
        let headphones_left_5_pin = gpioa.pa2.into_push_pull_output();

        let headphones_right_0_pin = gpioa.pa3.into_push_pull_output();
        let headphones_right_1_pin = gpioa.pa4.into_push_pull_output();
        let headphones_right_2_pin = gpioa.pa5.into_push_pull_output();
        let headphones_right_3_pin = gpioa.pa6.into_push_pull_output();
        let headphones_right_4_pin = gpioa.pa7.into_push_pull_output();
        let headphones_right_5_pin = gpiob.pb0.into_push_pull_output();

        let speakers_left_0_pin = gpiob.pb7.into_push_pull_output();
        let speakers_left_1_pin = gpiob.pb5.into_push_pull_output();
        let speakers_left_2_pin = gpiob.pb3.into_push_pull_output();
        let speakers_left_3_pin = gpiob.pb10.into_push_pull_output();
        let speakers_left_4_pin = gpiob.pb2.into_push_pull_output();
        let speakers_left_5_pin = gpiob.pb1.into_push_pull_output();

        let speakers_right_0_pin = gpioa.pa11.into_push_pull_output();
        let speakers_right_1_pin = gpioa.pa10.into_push_pull_output();
        let speakers_right_2_pin = gpiob.pb15.into_push_pull_output();
        let speakers_right_3_pin = gpiob.pb14.into_push_pull_output();
        let speakers_right_4_pin = gpiob.pb13.into_push_pull_output();
        let speakers_right_5_pin = gpiob.pb12.into_push_pull_output();

        headphones::spawn_after(REFRESH.micros()).unwrap();
        speakers::spawn_after(REFRESH.micros()).unwrap();

        (
            Shared {
                headphones_left: 0,
                headphones_right: 0,
                speakers_left: 0,
                speakers_right: 0,
            },
            Local {
                headphones_left_pwm,
                headphones_right_pwm,
                speakers_left_pwm,
                speakers_right_pwm,
                headphones_leds_0_pin: (headphones_left_0_pin, headphones_right_0_pin),
                headphones_leds_1_pin: (headphones_left_1_pin, headphones_right_1_pin),
                headphones_leds_2_pin: (headphones_left_2_pin, headphones_right_2_pin),
                headphones_leds_3_pin: (headphones_left_3_pin, headphones_right_3_pin),
                headphones_leds_4_pin: (headphones_left_4_pin, headphones_right_4_pin),
                headphones_leds_5_pin: (headphones_left_5_pin, headphones_right_5_pin),
                speakers_leds_0_pin: (speakers_left_0_pin, speakers_right_0_pin),
                speakers_leds_1_pin: (speakers_left_1_pin, speakers_right_1_pin),
                speakers_leds_2_pin: (speakers_left_2_pin, speakers_right_2_pin),
                speakers_leds_3_pin: (speakers_left_3_pin, speakers_right_3_pin),
                speakers_leds_4_pin: (speakers_left_4_pin, speakers_right_4_pin),
                speakers_leds_5_pin: (speakers_left_5_pin, speakers_right_5_pin),
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM1_CC, shared = [headphones_left], local = [headphones_left_pwm])]
    fn headphones_left(mut cx: headphones_left::Context) {
        let value = cx.local.headphones_left_pwm.get_duty_cycle();

        cx.shared.headphones_left.lock(|left| {
            if value != 100.0 {
                *left = match value {
                    x if x > 66.0 => 6,
                    x if x > 55.0 => 5,
                    x if x > 44.0 => 4,
                    x if x > 33.0 => 3,
                    x if x > 22.0 => 2,
                    x if x > 11.0 => 1,
                    _ => 0,
                };
            }
        });
    }

    #[task(binds = TIM2, shared = [headphones_right], local = [headphones_right_pwm])]
    fn headphones_right(mut cx: headphones_right::Context) {
        let value = cx.local.headphones_right_pwm.get_duty_cycle();

        cx.shared.headphones_right.lock(|right| {
            if value != 100.0 {
                *right = match value {
                    x if x > 99.0 => 0,
                    x if x > 66.0 => 6,
                    x if x > 55.0 => 5,
                    x if x > 44.0 => 4,
                    x if x > 33.0 => 3,
                    x if x > 22.0 => 2,
                    x if x > 11.0 => 1,
                    _ => 0,
                };
            }
        });
    }

    #[task(binds = TIM3, shared = [speakers_left], local = [speakers_left_pwm])]
    fn speakers_left(mut cx: speakers_left::Context) {
        let value = cx.local.speakers_left_pwm.get_duty_cycle();

        cx.shared.speakers_left.lock(|left| {
            if value != 100.0 {
                *left = match value {
                    x if x > 66.0 => 6,
                    x if x > 55.0 => 5,
                    x if x > 44.0 => 4,
                    x if x > 33.0 => 3,
                    x if x > 22.0 => 2,
                    x if x > 11.0 => 1,
                    _ => 0,
                };
            }
        });
    }

    #[task(binds = TIM4, shared = [speakers_right], local = [speakers_right_pwm])]
    fn speakers_right(mut cx: speakers_right::Context) {
        let value = cx.local.speakers_right_pwm.get_duty_cycle();

        cx.shared.speakers_right.lock(|right| {
            if value != 100.0 {
                *right = match value {
                    x if x > 99.0 => 0,
                    x if x > 66.0 => 6,
                    x if x > 55.0 => 5,
                    x if x > 44.0 => 4,
                    x if x > 33.0 => 3,
                    x if x > 22.0 => 2,
                    x if x > 11.0 => 1,
                    _ => 0,
                };
            }
        });
    }

    #[task(shared = [headphones_left, headphones_right], local = [headphones_leds_0_pin, headphones_leds_1_pin, headphones_leds_2_pin, headphones_leds_3_pin, headphones_leds_4_pin, headphones_leds_5_pin])]
    fn headphones(cx: headphones::Context) {
        let mut leds = [
            (
                &mut cx.local.headphones_leds_0_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_0_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.headphones_leds_1_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_1_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.headphones_leds_2_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_2_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.headphones_leds_3_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_3_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.headphones_leds_4_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_4_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.headphones_leds_5_pin.0 as &mut Led,
                &mut cx.local.headphones_leds_5_pin.1 as &mut Led,
            ),
        ];

        (cx.shared.headphones_left, cx.shared.headphones_right).lock(|left, right| {
            for (index, (left_pin, right_pin)) in leds.iter_mut().enumerate() {
                if *left > index as u8 {
                    left_pin.set_high().unwrap();
                } else {
                    left_pin.set_low().unwrap();
                }

                if *right > index as u8 {
                    right_pin.set_high().unwrap();
                } else {
                    right_pin.set_low().unwrap();
                }
            }
        });

        headphones::spawn_after(REFRESH.micros()).unwrap();
    }

    #[task(shared = [speakers_left, speakers_right], local = [speakers_leds_0_pin, speakers_leds_1_pin, speakers_leds_2_pin, speakers_leds_3_pin, speakers_leds_4_pin, speakers_leds_5_pin])]
    fn speakers(cx: speakers::Context) {
        let mut leds = [
            (
                &mut cx.local.speakers_leds_0_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_0_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.speakers_leds_1_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_1_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.speakers_leds_2_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_2_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.speakers_leds_3_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_3_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.speakers_leds_4_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_4_pin.1 as &mut Led,
            ),
            (
                &mut cx.local.speakers_leds_5_pin.0 as &mut Led,
                &mut cx.local.speakers_leds_5_pin.1 as &mut Led,
            ),
        ];

        (cx.shared.speakers_left, cx.shared.speakers_right).lock(|left, right| {
            for (index, (left_pin, right_pin)) in leds.iter_mut().enumerate() {
                if *left > index as u8 {
                    left_pin.set_high().unwrap();
                } else {
                    left_pin.set_low().unwrap();
                }

                if *right > index as u8 {
                    right_pin.set_high().unwrap();
                } else {
                    right_pin.set_low().unwrap();
                }
            }
        });

        speakers::spawn_after(REFRESH.micros()).unwrap();
    }
}
