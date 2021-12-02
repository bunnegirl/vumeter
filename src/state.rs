use crate::leds::*;
pub use Message::*;
pub use Mode::*;
pub use State::*;

type Resources<'a> = crate::app::dispatch::LocalResources<'a>;

fn duty_cycle_to_level(value: f32) -> LedLevel {
    match value {
        x if x > 99.0 => None,
        x if x > 66.0 => Some(5),
        x if x > 55.0 => Some(4),
        x if x > 44.0 => Some(3),
        x if x > 33.0 => Some(2),
        x if x > 22.0 => Some(1),
        x if x > 11.0 => Some(0),
        _ => None,
    }
}

fn update_levels(left: f32, right: f32, res: &mut Resources) {
    res.left_leds.set_level(duty_cycle_to_level(left));
    res.right_leds.set_level(duty_cycle_to_level(right));
}

fn update_peaks(left: f32, right: f32, res: &mut Resources) {
    res.left_leds.set_peak(duty_cycle_to_level(left));
    res.right_leds.set_peak(duty_cycle_to_level(right));
}

fn mute(high: bool, res: &mut Resources) {
    res.mute_output.set_low();

    if high {
        res.left_leds.set_peak(Some(5));
        res.right_leds.set_peak(Some(5));
    } else {
        res.left_leds.clear();
        res.right_leds.clear();
    }
}

fn unmute(res: &mut Resources) {
    res.mute_output.set_high();
    res.left_leds.clear();
    res.right_leds.clear();
}

pub enum Message {
    Animate(u32),
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
                mute(true, res);

                Some(Muted { high: true, mode: *mode })
            }

            // animate mute indicator
            (Muted { mut high, mode }, Animate(_)) => {
                high = !high;

                mute(high, res);

                Some(Muted { high, mode: *mode })
            }

            _ => None,
        }
    }
}
