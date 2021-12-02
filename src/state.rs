use crate::leds::*;
pub use Message::*;
pub use Mode::*;
pub use State::*;

type Resources<'a> = crate::app::dispatch::LocalResources<'a>;

fn update_levels(left: f32, right: f32, res: &mut Resources) {
    res.left_leds.set(LedPattern::from(left));
    res.right_leds.set(LedPattern::from(right));
}

fn update_peaks(left: f32, right: f32, res: &mut Resources) {
    res.left_leds.set(LedPattern::from(left).peak());
    res.right_leds.set(LedPattern::from(right).peak());
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

pub enum Message {
    AnimationFrame(u32),
    ToggleMute,
    ToggleMode,
    Update(f32, f32),
}

#[derive(Clone, Copy)]
pub enum Mode {
    Levels,
    Peaks,
}

#[derive(Clone, Copy)]
pub enum State {
    Show { mode: Mode },
    Muted { mode: Mode, high: bool },
}

impl State {
    pub fn dispatch(
        &mut self,
        res: &mut crate::app::dispatch::LocalResources,
        msg: Message,
    ) -> Option<Self> {
        match (self, msg) {
            // update levels
            (Show { mode: Levels }, Update(left, right)) => {
                update_levels(left, right, res);

                None
            }

            // switch to peaks mode
            (Show { mode: Levels }, ToggleMode) => Some(Show { mode: Peaks }),

            // update peaks
            (Show { mode: Peaks }, Update(left, right)) => {
                update_peaks(left, right, res);

                None
            }

            // switch to levels mode
            (Show { mode: Peaks }, ToggleMode) => Some(Show { mode: Levels }),

            // unmute audio
            (Muted { mode, .. }, ToggleMute) => {
                unmute(res);

                Some(Show { mode: *mode })
            }

            // mute audio
            (Show { mode }, ToggleMute) => {
                mute(false, res);

                Some(Muted { high: false, mode: *mode })
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
