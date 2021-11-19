#![no_main]
#![no_std]
#![feature(trait_alias)]

mod mono_timer;

use panic_semihosting as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use crate::mono_timer::MonoTimer;
    use core::convert::Infallible;
    use fugit::{ExtU32};
    use heapless::spsc::Queue;
    use rtt_target::{rprintln, rtt_init_print};
    use stm32f4xx_hal::{
        adc::{
            config::{AdcConfig, SampleTime},
            Adc,
        },
        gpio::{gpioa::*, gpiob::*, Analog, Output, PushPull},
        hal::digital::v2::OutputPin,
        pac,
        pac::ADC1,
        prelude::*,
    };
    use systick_monotonic::*;

    const SAMPLES: usize = 200;
    const PEAK_LEVEL: f32 = 89.0;
    const PLUS_6_LEVEL: f32 = 55.0;
    const NOMINAL_LEVEL: f32 = 34.0;
    const MINUS_6_LEVEL: f32 = 21.0;
    const MINUS_12_LEVEL: f32 = 13.0;
    const MINUS_24_LEVEL: f32 = 8.0;

    type Led = dyn OutputPin<Error = Infallible>;

    #[monotonic(binds = TIM5, default = true)]
    type Timer = MonoTimer<pac::TIM5, 2_000_000>;

    #[shared]
    struct Shared {
        left_data: Queue<u16, SAMPLES>,
        right_data: Queue<u16, SAMPLES>,
    }

    #[local]
    struct Local {
        adc: Adc<ADC1>,
        left_level: PA0<Analog>,
        right_level: PA1<Analog>,
        level_0_leds: (PB3<Output<PushPull>>, PB0<Output<PushPull>>),
        level_1_leds: (PB4<Output<PushPull>>, PB1<Output<PushPull>>),
        level_2_leds: (PB5<Output<PushPull>>, PB2<Output<PushPull>>),
        level_3_leds: (PB6<Output<PushPull>>, PB10<Output<PushPull>>),
        level_4_leds: (PB7<Output<PushPull>>, PB12<Output<PushPull>>),
        level_5_leds: (PB8<Output<PushPull>>, PB13<Output<PushPull>>),
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let adc = Adc::adc1(cx.device.ADC1, true, AdcConfig::default());
        let mono = Timer::new(cx.device.TIM5, &clocks);

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();

        let left_data = Queue::new();
        let left_level = gpioa.pa0.into_analog();
        let right_data = Queue::new();
        let right_level = gpioa.pa1.into_analog();

        let level_0_leds = (
            gpiob.pb3.into_push_pull_output(),
            gpiob.pb0.into_push_pull_output(),
        );
        let level_1_leds = (
            gpiob.pb4.into_push_pull_output(),
            gpiob.pb1.into_push_pull_output(),
        );
        let level_2_leds = (
            gpiob.pb5.into_push_pull_output(),
            gpiob.pb2.into_push_pull_output(),
        );
        let level_3_leds = (
            gpiob.pb6.into_push_pull_output(),
            gpiob.pb10.into_push_pull_output(),
        );
        let level_4_leds = (
            gpiob.pb7.into_push_pull_output(),
            gpiob.pb12.into_push_pull_output(),
        );
        let level_5_leds = (
            gpiob.pb8.into_push_pull_output(),
            gpiob.pb13.into_push_pull_output(),
        );

        update::spawn().unwrap();

        (
            Shared {
                left_data,
                right_data,
            },
            Local {
                adc,
                left_level,
                right_level,
                level_0_leds,
                level_1_leds,
                level_2_leds,
                level_3_leds,
                level_4_leds,
                level_5_leds,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(shared = [left_data, right_data], local = [adc, left_level, right_level])]
    fn idle(cx: idle::Context) -> ! {
        let adc = cx.local.adc;
        let left_level = cx.local.left_level;
        let mut left_data = cx.shared.left_data;
        let right_level = cx.local.right_level;
        let mut right_data = cx.shared.right_data;

        loop {
            let left_sample = adc.convert(left_level, SampleTime::Cycles_28);
            let left_val = adc.sample_to_millivolts(left_sample);
            let right_sample = adc.convert(right_level, SampleTime::Cycles_28);
            let right_val = adc.sample_to_millivolts(right_sample);

            left_data.lock(|left_data| {
                if left_data.is_full() {
                    left_data.dequeue().unwrap();
                }

                left_data.enqueue(left_val).unwrap();
            });

            right_data.lock(|right_data| {
                if right_data.is_full() {
                    right_data.dequeue().unwrap();
                }

                right_data.enqueue(right_val).unwrap();
            });

            cortex_m::asm::nop();
        }
    }

    #[task(shared = [left_data, right_data], local = [level_0_leds, level_1_leds, level_2_leds, level_3_leds, level_4_leds, level_5_leds])]
    fn update(mut cx: update::Context) {
        update::spawn_after(50u32.millis()).unwrap();

        let left_val = cx.shared.left_data.lock(|left_data| {
            let val: f32 = left_data.iter().fold(0.0, |val, item| val + (*item as f32));

            val / (SAMPLES as f32)
        });

        let right_val = cx.shared.right_data.lock(|right_data| {
            let val: f32 = right_data
                .iter()
                .fold(0.0, |val, item| val + (*item as f32));

            val / (SAMPLES as f32)
        });

        if left_val > PEAK_LEVEL || right_val > PEAK_LEVEL {
            rprintln!("peak: {:>6.2} {:>6.2}", left_val, right_val);
        }

        let left_level = if left_val > PEAK_LEVEL {
            6
        } else if left_val > PLUS_6_LEVEL {
            5
        } else if left_val > NOMINAL_LEVEL {
            4
        } else if left_val > MINUS_6_LEVEL {
            3
        } else if left_val > MINUS_12_LEVEL {
            2
        } else if left_val > MINUS_24_LEVEL {
            1
        } else {
            0
        };

        let right_level = if right_val > PEAK_LEVEL {
            6
        } else if right_val > PLUS_6_LEVEL {
            5
        } else if right_val > NOMINAL_LEVEL {
            4
        } else if right_val > MINUS_6_LEVEL {
            3
        } else if right_val > MINUS_12_LEVEL {
            2
        } else if right_val > MINUS_24_LEVEL {
            1
        } else {
            0
        };

        let mut leds = [
            (
                &mut cx.local.level_0_leds.0 as &mut Led,
                &mut cx.local.level_0_leds.1 as &mut Led,
            ),
            (
                &mut cx.local.level_1_leds.0 as &mut Led,
                &mut cx.local.level_1_leds.1 as &mut Led,
            ),
            (
                &mut cx.local.level_2_leds.0 as &mut Led,
                &mut cx.local.level_2_leds.1 as &mut Led,
            ),
            (
                &mut cx.local.level_3_leds.0 as &mut Led,
                &mut cx.local.level_3_leds.1 as &mut Led,
            ),
            (
                &mut cx.local.level_4_leds.0 as &mut Led,
                &mut cx.local.level_4_leds.1 as &mut Led,
            ),
            (
                &mut cx.local.level_5_leds.0 as &mut Led,
                &mut cx.local.level_5_leds.1 as &mut Led,
            ),
        ];

        for (index, (left, right)) in leds.iter_mut().enumerate() {
            if left_level > index {
                left.set_high().unwrap();
            } else {
                left.set_low().unwrap();
            }

            if right_level > index {
                right.set_high().unwrap();
            } else {
                right.set_low().unwrap();
            }
        }
    }
}
