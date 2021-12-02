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
use fugit::ExtU32;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    gpio::{gpioa::*, gpiob::*, *},
    pac,
    prelude::*,
    pwm_input::PwmInput,
    stm32::{TIM2, TIM3},
    timer::Timer as HalTimer,
};

const PWM_FREQUENCY: u32 = 6;
const ANIMATION_INTERVAL: u32 = 100;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        cortex_m::asm::nop();
    }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI2, EXTI3, EXTI4])]
mod app {
    use super::*;

    #[monotonic(binds = TIM5, default = true)]
    type Timer = MonoTimer<pac::TIM5, 8_000_000>;

    #[shared]
    struct Shared {
        state: State,
        animation_enabled: bool,
    }

    #[local]
    struct Local {
        mute_toggle: PA0<Input<PullUp>>,
        mute_output: PB0<Output<PushPull>>,
        mode_toggle: PA1<Input<PullUp>>,
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

        let mut mute_toggle = gpioa.pa0.into_pull_up_input();

        mute_toggle.make_interrupt_source(&mut syscfg);
        mute_toggle.enable_interrupt(&mut cx.device.EXTI);
        mute_toggle.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        let mut mute_output = gpiob.pb0.into_push_pull_output();

        mute_output.set_high();

        let mut mode_toggle = gpioa.pa1.into_pull_up_input();

        mode_toggle.make_interrupt_source(&mut syscfg);
        mode_toggle.enable_interrupt(&mut cx.device.EXTI);
        mode_toggle.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

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

        (
            Shared {
                state: State::Show { mode: Levels },
                animation_enabled: false,
            },
            Local {
                mute_toggle,
                mute_output,
                mode_toggle,
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

    #[task(
        shared = [state],
        local = [mute_output, left_leds, right_leds],
        priority = 5
    )]
    fn dispatch(mut cx: dispatch::Context, msg: Message) {
        let mut state = cx.shared.state.lock(|state| *state);

        if let Some(new_state) = state.dispatch(&mut cx.local, msg) {
            cx.shared.state.lock(|state| *state = new_state);
        }
    }

    #[task(
        shared = [animation_enabled]
    )]
    fn start_animation(mut cx: start_animation::Context) {
        cx.shared.animation_enabled.lock(|enabled| {
            *enabled = true;
        });

        animation_frame::spawn_after(ANIMATION_INTERVAL.millis()).ok();
    }

    #[task(
        shared = [animation_enabled],
        local = [counter: u32 = 0],
        priority = 6
    )]
    fn animation_frame(mut cx: animation_frame::Context) {
        let enabled = cx.shared.animation_enabled.lock(|enabled| *enabled);

        if enabled {
            animation_frame::spawn_after(ANIMATION_INTERVAL.millis()).ok();
        }

        dispatch::spawn(AnimationFrame(*cx.local.counter)).ok();

        *cx.local.counter += 1;
    }

    #[task(
        shared = [animation_enabled]
    )]
    fn stop_animation(mut cx: stop_animation::Context) {
        cx.shared.animation_enabled.lock(|enabled| {
            *enabled = false;
        });
    }

    #[task(
        binds = EXTI0,
        local = [
            mute_toggle,
            debouncer: Debounce<150> = Debounce(None)
        ],
        priority = 9
    )]
    fn toggle_mute(cx: toggle_mute::Context) {
        cx.local.mute_toggle.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            dispatch::spawn(ToggleMute).ok();

            cx.local.debouncer.reset();
        } else {
            cx.local.debouncer.update();
        }
    }

    #[task(
        binds = EXTI1,
        local = [
            mode_toggle,
            debouncer: Debounce<150> = Debounce(None)
        ],
        priority = 9
    )]
    fn toggle_mode(cx: toggle_mode::Context) {
        cx.local.mode_toggle.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            dispatch::spawn(ToggleMode).ok();

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

        dispatch::spawn(Update(left, right)).ok();
    }
}
