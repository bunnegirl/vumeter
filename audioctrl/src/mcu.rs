use crate::bus::*;
use crate::debounce::*;
use crate::meter::*;
use crate::mono_timer::MonoTimer;
use crate::state::*;
use fugit::ExtU32;
use heapless::HistoryBuffer;
use rtic::Mutex;
use rtt_target::*;
use stm32h7xx_hal::{
    gpio::{gpiod::*, gpioe::*, *},
    hal::digital::v2::*,
    pac,
    prelude::*,
    rcc::rec::AdcClkSel,
};

pub use app::*;

struct Local {
    device_in: PD14<Input<PullUp>>,
    device_out: PD15<Output<PushPull>>,
    mute_in: PD12<Input<PullUp>>,
    mute_out: PD13<Output<PushPull>>,
    power_in: PD10<Input<PullUp>>,
    power_out: PE0<Output<PushPull>>,
    power_led: PD11<Output<PushPull>>,
    mon_clock: PD0<Input<PullUp>>,
    mon_left: PD2<Input<PullUp>>,
    mon_right: PD4<Input<PullUp>>,
    meter: Meter<Output<PushPull>>,
}

fn clock_send(_: clock_send::Context, count: u32) {
    clock_send::spawn_after(10.millis(), if count == u32::MAX { 0 } else { count + 1 }).ok();

    Clock(count).send();
}

fn message_dispatch(cx: message_dispatch::Context) {
    let message_dispatch::SharedResources { mut state } = cx.shared;

    if let Some(bus) = Q.dequeue() {
        match bus {
            ToMcu(msg) => {
                message_recv::spawn(msg).ok();
            }
            ToState(msg) => {
                state.lock(|state| {
                    *state = modify_state(*state, msg);
                });
            }
        }
    }
}

fn message_recv(cx: message_recv::Context, msg: McuMsg) {
    match msg {
        SetDevice(Headphones) => {
            cx.local.device_out.set_low().ok();
        }
        SetDevice(Speakers) => {
            cx.local.device_out.set_high().ok();
        }
        SetMute(Muted) => {
            cx.local.mute_out.set_high().ok();
        }
        SetMute(Unmuted) => {
            cx.local.mute_out.set_low().ok();
        }
        SetPower(Booting) => {
            panic!("undefined behaviour");
        }
        SetPower(PowerOn) => {
            cx.local.power_led.set_high().ok();
            cx.local.power_out.set_high().ok();
        }
        SetPower(PowerOff) => {
            cx.local.power_led.set_low().ok();
            cx.local.power_out.set_low().ok();
        }
        SetMeter(Patterns(left, right)) => {
            cx.local.meter.set(left, right);
        }
    }
}

fn ctrl_change(cx: ctrl_change::Context) {
    let ctrl_change::LocalResources {
        device_in,
        power_in,
        mute_in,
        debouncers,
    } = cx.local;

    device_in.clear_interrupt_pending_bit();
    power_in.clear_interrupt_pending_bit();
    mute_in.clear_interrupt_pending_bit();

    let ctrl = (
        device_in.is_high().unwrap(),
        power_in.is_high().unwrap(),
        mute_in.is_high().unwrap(),
    );

    let msg = match ctrl {
        (true, false, false) => Some((1, ToggleDevice, 250)),
        (false, true, false) => Some((2, TogglePower, 500)),
        (false, false, true) => Some((3, ToggleMute, 250)),
        _ => None,
    };

    if let Some((id, msg, delay)) = msg {
        if debouncers.is_ok(id) {
            msg.send();
        }

        debouncers.update(id, delay);
    }
}

fn level_change(cx: level_change::Context) {
    let level_change::LocalResources {
        mon_clock,
        mon_left,
        mon_right,
        capture_left,
        capture_right,
    } = cx.local;

    mon_clock.clear_interrupt_pending_bit();

    if capture_left.len() == 400 || capture_right.len() == 400 {
        let left = Level::from(capture_left.as_slice().iter().sum::<f32>() / 400.0);
        let right = Level::from(capture_right.as_slice().iter().sum::<f32>() / 400.0);

        UpdateMeter(Levels(left, right)).send();

        *capture_left = HistoryBuffer::new();
        *capture_right = HistoryBuffer::new();
    }

    fn set(is_high: bool) -> f32 {
        (is_high as u32) as f32
    }

    capture_left.write(set(mon_left.is_high().unwrap()));
    capture_right.write(set(mon_right.is_high().unwrap()));
}

#[rtic::app(device = stm32h7xx_hal::pac, dispatchers = [SPI1, SPI2, SPI3, SPI4])]
mod app {
    use super::*;

    #[monotonic(binds = TIM2, default = true)]
    type Timer = MonoTimer<pac::TIM2, 1_000_000>;

    #[shared]
    struct Shared {
        state: State,
    }

    #[local]
    struct Local {
        device_in: PD14<Input<PullUp>>,
        device_out: PD15<Output<PushPull>>,
        mute_in: PD12<Input<PullUp>>,
        mute_out: PD13<Output<PushPull>>,
        power_in: PD10<Input<PullUp>>,
        power_out: PE0<Output<PushPull>>,
        power_led: PD11<Output<PushPull>>,
        mon_clock: PD0<Input<PullUp>>,
        mon_left: PD2<Input<PullUp>>,
        mon_right: PD4<Input<PullUp>>,
        meter: Meter<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let mut syscfg = cx.device.SYSCFG;
        let mut exti = cx.device.EXTI;

        let pwr = cx.device.PWR.constrain();
        let pwrcfg = pwr.freeze();

        let rcc = cx.device.RCC.constrain();
        let mut ccdr = rcc
            .sys_ck(200.mhz())
            .per_ck(4.mhz())
            .freeze(pwrcfg, &syscfg);

        ccdr.peripheral.kernel_adc_clk_mux(AdcClkSel::PER);

        let mono = MonoTimer::new(cx.device.TIM2, &ccdr.clocks);

        let gpioa = cx.device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpioe = cx.device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiod = cx.device.GPIOD.split(ccdr.peripheral.GPIOD);

        let mut device_in = gpiod.pd14.into_pull_up_input();
        let device_out = gpiod.pd15.into_push_pull_output();

        device_in.make_interrupt_source(&mut syscfg);
        device_in.enable_interrupt(&mut exti);
        device_in.trigger_on_edge(&mut exti, Edge::Falling);

        let mut mute_in = gpiod.pd12.into_pull_up_input();
        let mut mute_out = gpiod.pd13.into_push_pull_output();

        mute_in.make_interrupt_source(&mut syscfg);
        mute_in.enable_interrupt(&mut exti);
        mute_in.trigger_on_edge(&mut exti, Edge::Falling);
        mute_out.set_low().ok();

        let mut power_in = gpiod.pd10.into_pull_up_input();
        let power_out = gpioe.pe0.into_push_pull_output();
        let power_led = gpiod.pd11.into_push_pull_output();

        power_in.make_interrupt_source(&mut syscfg);
        power_in.enable_interrupt(&mut exti);
        power_in.trigger_on_edge(&mut exti, Edge::Falling);

        let mut mon_clock = gpiod.pd0.into_pull_up_input();
        let mon_left = gpiod.pd2.into_pull_up_input();
        let mon_right = gpiod.pd4.into_pull_up_input();

        mon_clock.make_interrupt_source(&mut syscfg);
        mon_clock.trigger_on_edge(&mut exti, Edge::RisingFalling);
        mon_clock.enable_interrupt(&mut exti);

        let meter: Meter<Output<PushPull>> = [
            Clip.pins(
                LeftPin::Clip(gpioa.pa12.into_push_pull_output()),
                RightPin::Clip(gpioe.pe2.into_push_pull_output()),
            ),
            Plus6.pins(
                LeftPin::Plus6(gpioa.pa10.into_push_pull_output()),
                RightPin::Plus6(gpioe.pe4.into_push_pull_output()),
            ),
            Nominal.pins(
                LeftPin::Nominal(gpioa.pa8.into_push_pull_output()),
                RightPin::Nominal(gpioe.pe6.into_push_pull_output()),
            ),
            Minus6.pins(
                LeftPin::Minus6(gpioa.pa5.into_push_pull_output()),
                RightPin::Minus6(gpioe.pe11.into_push_pull_output()),
            ),
            Minus12.pins(
                LeftPin::Minus12(gpioa.pa3.into_push_pull_output()),
                RightPin::Minus12(gpioe.pe13.into_push_pull_output()),
            ),
            Minus18.pins(
                LeftPin::Minus18(gpioa.pa1.into_push_pull_output()),
                RightPin::Minus18(gpioe.pe15.into_push_pull_output()),
            ),
            Minus24.pins(
                LeftPin::Minus24(gpioa.pa4.into_push_pull_output()),
                RightPin::Minus24(gpioe.pe10.into_push_pull_output()),
            ),
            Minus30.pins(
                LeftPin::Minus30(gpioa.pa2.into_push_pull_output()),
                RightPin::Minus30(gpioe.pe12.into_push_pull_output()),
            ),
            Minus36.pins(
                LeftPin::Minus36(gpioa.pa0.into_push_pull_output()),
                RightPin::Minus36(gpioe.pe14.into_push_pull_output()),
            ),
        ];

        clock_send::spawn_after(10.millis(), 0).ok();

        Initialise.send();

        (
            Shared {
                state: State::new(),
            },
            Local {
                device_in,
                device_out,
                mute_out,
                mute_in,
                power_out,
                power_in,
                power_led,
                mon_clock,
                mon_left,
                mon_right,
                meter,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            message_dispatch::spawn().ok();
        }
    }

    extern "Rust" {
        #[task]
        fn clock_send(cx: clock_send::Context, count: u32);

        #[task(
            shared = [state]
        )]
        fn message_dispatch(cx: message_dispatch::Context);

        #[task(
            local = [
                device_out,
                power_out,
                power_led,
                mute_out,
                meter,
            ]
        )]
        fn message_recv(cx: message_recv::Context, msg: McuMsg);

        #[task(
            binds = EXTI15_10,
            local = [
                device_in,
                power_in,
                mute_in,
                debouncers: Debouncers<3> = Debouncers::new(),
            ],
            priority = 3
        )]
        fn ctrl_change(cx: ctrl_change::Context);

        #[task(
            binds = EXTI0,
            local = [
                mon_clock,
                mon_left,
                mon_right,
                capture_left: HistoryBuffer<f32, 400> = HistoryBuffer::new(),
                capture_right: HistoryBuffer<f32, 400> = HistoryBuffer::new(),
            ],
            priority = 2,
        )]
        fn level_change(cx: level_change::Context);
    }
}
