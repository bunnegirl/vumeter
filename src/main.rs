#![no_main]
#![no_std]
#![feature(trait_alias)]

mod debounce;
mod leds;
mod mono_timer;
mod state;

use crate::debounce::*;
use crate::leds::*;
use crate::mono_timer::MonoTimer;
use crate::state::*;
use core::panic::PanicInfo;
use rtt_target::{rprintln, rtt_init_print};
use fugit::{ExtU32};
use stm32f4xx_hal::{
    gpio::{gpioa::*, gpiob::*, *},
    pac,
    prelude::*,
    pwm_input::PwmInput,
    stm32::{TIM2, TIM3},
    timer::Timer as HalTimer,
};

const PWM_FREQUENCY: u32 = 6;
const ANIMATION_INTERVAL: u32 = 2;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        cortex_m::asm::nop();
    }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI3, EXTI4])]
mod app {
    use super::*;

    #[monotonic(binds = TIM5, default = true)]
    type Timer = MonoTimer<pac::TIM5, 8_000_000>;

    #[shared]
    struct Shared {
        state: State,
    }

    #[local]
    struct Local {
        mute_input: PA0<Input<PullUp>>,
        left_input: PwmInput<TIM2, PA5<Alternate<1>>>,
        right_input: PwmInput<TIM3, PA6<Alternate<2>>>,
        left_leds: Leds<
            PB12<Output<PushPull>>,
            PB13<Output<PushPull>>,
            PB14<Output<PushPull>>,
            PB15<Output<PushPull>>,
            PA8<Output<PushPull>>,
            PA9<Output<PushPull>>,
        >,
        right_leds: Leds<
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
            PB7<Output<PushPull>>,
            PB8<Output<PushPull>>,
        >,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let mut syscfg = cx.device.SYSCFG.constrain();
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

        let mut mute_input = gpioa.pa0.into_pull_up_input();

        mute_input.make_interrupt_source(&mut syscfg);
        mute_input.enable_interrupt(&mut cx.device.EXTI);
        mute_input.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        let left_input = HalTimer::new(cx.device.TIM2, &clocks)
            .pwm_input(PWM_FREQUENCY.khz(), gpioa.pa5.into_alternate());

        let left_leds = Leds(
            gpiob.pb12.into_push_pull_output(),
            gpiob.pb13.into_push_pull_output(),
            gpiob.pb14.into_push_pull_output(),
            gpiob.pb15.into_push_pull_output(),
            gpioa.pa8.into_push_pull_output(),
            gpioa.pa9.into_push_pull_output(),
        );

        let right_input = HalTimer::new(cx.device.TIM3, &clocks)
            .pwm_input(PWM_FREQUENCY.khz(), gpioa.pa6.into_alternate());

        let right_leds = Leds(
            gpiob.pb3.into_push_pull_output(),
            gpiob.pb4.into_push_pull_output(),
            gpiob.pb5.into_push_pull_output(),
            gpiob.pb6.into_push_pull_output(),
            gpiob.pb7.into_push_pull_output(),
            gpiob.pb8.into_push_pull_output(),
        );

        animate::spawn().ok();

        (
            Shared { state: State::Monitor },
            Local {
                mute_input,
                left_leds,
                left_input,
                right_input,
                right_leds,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(shared = [state], local = [left_leds, right_leds])]
    fn dispatch(mut cx: dispatch::Context, msg: Message) {
        let mut state = cx.shared.state.lock(|state| *state);

        if let Some(new_state) = state.dispatch(&mut cx, msg) {
            cx.shared.state.lock(|state| *state = new_state);
        }
    }

    #[task(local = [interval: u32 = 0], priority = 2)]
    fn animate(cx: animate::Context) {
        *cx.local.interval += 1;

        animate::spawn_after(ANIMATION_INTERVAL.secs()).ok();
        dispatch::spawn(Animate(*cx.local.interval)).ok();
    }

    #[task(binds = EXTI0, local = [
        mute_input,
        debouncer: Debounce<150> = Debounce(None)
    ])]
    fn toggle_mute(cx: toggle_mute::Context) {
        cx.local.mute_input.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            dispatch::spawn(ToggleMute).ok();

            cx.local.debouncer.reset();
        } else {
            cx.local.debouncer.update();
        }
    }

    #[task(
        binds = TIM2,
        local = [left_input, right_input,]
    )]
    fn monitor_inputs(cx: monitor_inputs::Context) {
        let left = cx.local.left_input.get_duty_cycle();
        let right = cx.local.right_input.get_duty_cycle();

        dispatch::spawn(SetLevels(left, right)).ok();
    }
}
