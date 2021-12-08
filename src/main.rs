#![no_main]
#![no_std]

mod buffer;
mod debounce;
mod leds;
mod mono_timer;
mod state;

use crate::buffer::*;
use crate::debounce::*;
use crate::leds::*;
use crate::mono_timer::MonoTimer;
use crate::state::*;
use core::panic::PanicInfo;
use fugit::{ExtU32, Instant};
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    gpio::{gpioa::*, gpiob::*, *},
    pac,
    prelude::*,
    pwm_input::PwmInput,
    stm32::{TIM2, TIM3},
    timer::Timer as HalTimer,
};

/// frequency of dsp board's pwm signal (in kilohertz)
const PWM_FREQUENCY: u32 = 12;

/// how fast should animation frames play (in milliseconds)
const ANIMATION_INTERVAL: u32 = 100;

/// how frequently should the display update (in milliseconds)
const DISPLAY_UPDATE: u32 = 30;

/// length of time (in seconds) before idle animation starts
/// playing when no signal is detected.
const DISPLAY_TIMEOUT: u32 = 30;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        cortex_m::asm::nop();
    }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [SPI1, SPI2, SPI3])]
mod app {
    use super::*;

    #[monotonic(binds = TIM5, default = true)]
    type Timer = MonoTimer<pac::TIM5, 8_000_000>;

    #[shared]
    struct Shared {
        state: State,
        animation_enabled: bool,
        buffer: (HistoryBuffer<f32, 500>, HistoryBuffer<f32, 500>),
        timeout: Instant<u32, 1_u32, 8000000_u32>,
    }

    #[local]
    struct Local {
        toggle_mute: PA0<Input<PullUp>>,
        mute_output: PA7<Output<PushPull>>,
        toggle_meter: PA1<Input<PullUp>>,
        toggle_mode: PA2<Input<PullUp>>,
        mode_output: PB0<Output<PushPull>>,
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

        let mut toggle_mute = gpioa.pa0.into_pull_up_input();

        toggle_mute.make_interrupt_source(&mut syscfg);
        toggle_mute.enable_interrupt(&mut cx.device.EXTI);
        toggle_mute.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        let mut mute_output = gpioa.pa7.into_push_pull_output();

        mute_output.set_high();

        let mut toggle_meter = gpioa.pa1.into_pull_up_input();

        toggle_meter.make_interrupt_source(&mut syscfg);
        toggle_meter.enable_interrupt(&mut cx.device.EXTI);
        toggle_meter.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        let mut toggle_mode = gpioa.pa2.into_pull_up_input();

        toggle_mode.make_interrupt_source(&mut syscfg);
        toggle_mode.enable_interrupt(&mut cx.device.EXTI);
        toggle_mode.trigger_on_edge(&mut cx.device.EXTI, Edge::Falling);

        let mut mode_output = gpiob.pb0.into_push_pull_output();

        mode_output.set_high();

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

        update::spawn_after(DISPLAY_UPDATE.millis()).ok();
        timeout::spawn_after(DISPLAY_UPDATE.millis()).ok();
        message::spawn(Initialise).ok();

        (
            Shared {
                state: Uninitialised,
                animation_enabled: false,
                buffer: (Buffer::new(), Buffer::new()),
                timeout: monotonics::now(),
            },
            Local {
                toggle_mute,
                mute_output,
                toggle_meter,
                toggle_mode,
                mode_output,
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
    fn message(mut cx: message::Context, msg: Message) {
        let mut state = cx.shared.state.lock(|state| *state);

        if let Some(new_state) = state.message(&mut cx.local, msg) {
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

        animation_frame::spawn_after(ANIMATION_INTERVAL.millis(), 0).ok();
    }

    #[task(
        shared = [animation_enabled],
        priority = 6
    )]
    fn animation_frame(mut cx: animation_frame::Context, counter: u32) {
        let enabled = cx.shared.animation_enabled.lock(|enabled| *enabled);

        if enabled {
            let counter = if counter > 100 { 0 } else { counter };

            animation_frame::spawn_after(ANIMATION_INTERVAL.millis(), counter + 1).ok();
        }

        message::spawn(AnimationFrame(counter)).ok();
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
            toggle_mute,
            debouncer: Debounce<150> = Debounce(None)
        ],
        priority = 9
    )]
    fn toggle_mute(cx: toggle_mute::Context) {
        cx.local.toggle_mute.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            message::spawn(ToggleMute).ok();

            cx.local.debouncer.reset();
        } else {
            cx.local.debouncer.update();
        }
    }

    #[task(
        binds = EXTI1,
        local = [
            toggle_meter,
            debouncer: Debounce<150> = Debounce(None)
        ],
        priority = 9
    )]
    fn toggle_meter(cx: toggle_meter::Context) {
        cx.local.toggle_meter.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            message::spawn(ToggleMeter).ok();

            cx.local.debouncer.reset();
        } else {
            cx.local.debouncer.update();
        }
    }

    #[task(
        binds = EXTI2,
        local = [
            toggle_mode,
            debouncer: Debounce<150> = Debounce(None)
        ],
        priority = 9
    )]
    fn toggle_mode(cx: toggle_mode::Context) {
        cx.local.toggle_mode.clear_interrupt_pending_bit();

        if !cx.local.debouncer.is_bouncing() {
            message::spawn(ToggleMode).ok();

            cx.local.debouncer.reset();
        } else {
            cx.local.debouncer.update();
        }
    }

    #[task(
        binds = TIM2,
        shared = [buffer],
        local = [
            left_input,
            right_input,
        ],
        priority = 2
    )]
    fn buffer(mut cx: buffer::Context) {
        let left = cx.local.left_input.get_duty_cycle();
        let right = cx.local.right_input.get_duty_cycle();

        cx.shared.buffer.lock(|(left_buf, right_buf)| {
            left_buf.write(left);
            right_buf.write(right);
        });
    }

    #[task(shared = [buffer, timeout])]
    fn update(cx: update::Context) {
        update::spawn_after(DISPLAY_UPDATE.millis()).ok();

        (cx.shared.timeout, cx.shared.buffer).lock(|timeout, (left_buf, right_buf)| {
            let left = left_buf.avg();
            let right = right_buf.avg();

            if left > 0.0 || right > 0.0 {
                *timeout = monotonics::now();
            }

            message::spawn(Update(left, right)).ok();
        });
    }

    #[task(shared = [timeout])]
    fn timeout(mut cx: timeout::Context) {
        timeout::spawn_after(DISPLAY_UPDATE.millis()).ok();

        cx.shared.timeout.lock(|timeout| {
            if *timeout < monotonics::now() - DISPLAY_TIMEOUT.secs() {
                message::spawn(Timeout).ok();
            }
        });
    }
}
