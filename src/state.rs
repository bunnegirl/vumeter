use crate::leds::*;
pub use Message::*;
pub use MeterMode::*;
pub use State::*;

type Resources<'a> = crate::app::message::LocalResources<'a>;

fn idle_start() {
    crate::app::start_animation::spawn().ok();
}

fn idle_animation(level: u8, res: &mut Resources) {
    res.left_leds.set(LedPattern::from(level).peak());
    res.right_leds.set(LedPattern::from(level).peak());
}

fn idle_stop() {
    crate::app::stop_animation::spawn().ok();
}

fn meter(mode: MeterMode, left: f32, right: f32, res: &mut Resources) {
    let mut left = LedPattern::from(left);
    let mut right = LedPattern::from(right);

    if let Peaks = mode {
        left = left.peak();
        right = right.peak();
    }
    res.left_leds.set(left);
    res.right_leds.set(right);
}

fn mute(high: bool, res: &mut Resources) {
    crate::app::start_animation::spawn().ok();

    res.mute_output.set_low();

    if high {
        res.left_leds.set(LedPattern::from(5).peak());
        res.right_leds.set(LedPattern::from(5).peak());
    } else {
        res.left_leds.clear();
        res.right_leds.clear();
    }
}

fn unmute(res: &mut Resources) {
    crate::app::stop_animation::spawn().ok();

    res.mute_output.set_high();

    res.left_leds.clear();
    res.right_leds.clear();
}

#[derive(Debug)]
pub enum Message {
    Initialise,
    Timeout,
    AnimationFrame(u32),
    ToggleMute,
    ToggleMeter,
    ToggleMode,
    Update(f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum MeterMode {
    Levels,
    Peaks,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Uninitialised,
    Idle {
        mode: MeterMode,
        level: u8,
        up: bool,
    },
    Meter {
        mode: MeterMode,
    },
    Muted {
        mode: MeterMode,
        high: bool,
    },
}

impl State {
    pub fn message(&mut self, res: &mut Resources, msg: Message) -> Option<Self> {
        match (self, msg) {
            // set initial state
            (Uninitialised, Initialise) => Some(Meter { mode: Levels }),

            // start idle animation
            (Meter { mode }, Timeout) => {
                idle_start();

                Some(Idle {
                    mode: *mode,
                    level: 5,
                    up: false,
                })
            }

            // idle animation
            (Idle { mode, level, up }, AnimationFrame(_)) => {
                let (level, up) = if !*up {
                    // go back up
                    if *level == 0 {
                        (1, true)
                    } else {
                        (*level - 1, false)
                    }
                } else {
                    // go back down
                    if *level == 5 {
                        (4, false)
                    } else {
                        (*level + 1, true)
                    }
                };

                idle_animation(level, res);

                Some(Idle {
                    mode: *mode,
                    level,
                    up,
                })
            }

            // stop idle animation
            (Idle { .. }, Update(left, right)) => {
                if left > 0.0 || right > 0.0 {
                    idle_stop();
                    meter(Levels, left, right, res);
                    Some(Meter { mode: Levels })
                } else {
                    None
                }
            }

            // update meter
            (Meter { mode }, Update(left, right)) => {
                meter(*mode, left, right, res);

                None
            }

            // switch to peaks mode
            (Meter { mode: Levels }, ToggleMeter) => Some(Meter { mode: Peaks }),

            // switch to levels mode
            (Meter { mode: Peaks }, ToggleMeter) => Some(Meter { mode: Levels }),

            // unmute audio
            (Muted { mode, .. }, ToggleMute) => {
                unmute(res);

                Some(Meter { mode: *mode })
            }

            // mute audio
            (Meter { mode }, ToggleMute) => {
                mute(false, res);

                Some(Muted {
                    high: false,
                    mode: *mode,
                })
            }

            // mute audio
            (Idle { mode, .. }, ToggleMute) => {
                mute(false, res);

                Some(Muted {
                    high: false,
                    mode: *mode,
                })
            }

            // animate mute indicator
            (Muted { mut high, mode }, AnimationFrame(counter)) => {
                if counter % 20 == 0 {
                    high = !high;
                }

                mute(high, res);

                Some(Muted { high, mode: *mode })
            }

            _ => None,
        }
    }
}
