use crate::leds::*;
pub use Message::*;
pub use State::*;

const MUTED_INDICATOR: LedsPattern = [false, false, false, false, false, true];

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

pub enum Message {
    Animate(u32),
    ToggleMute,
    SetLevels(f32, f32),
}

#[derive(Clone, Copy)]
pub enum State {
    Monitor,
    Muted {high: bool},
}

impl State {
    pub fn dispatch(&mut self, cx: &mut crate::app::dispatch::Context, msg: Message) -> Option<Self> {
        match (self, msg) {
            (Monitor, SetLevels(left, right)) => {
                cx.local.left_leds.set_level(duty_cycle_to_level(left));
                cx.local.right_leds.set_level(duty_cycle_to_level(right));

                None
            }
            (Monitor, ToggleMute) => {
                cx.local.left_leds.set_pattern(MUTED_INDICATOR);
                cx.local.right_leds.set_pattern(MUTED_INDICATOR);

                Some(Muted { high: true })
            }
            (Muted { .. }, ToggleMute) => {
                cx.local.left_leds.reset_pattern(MUTED_INDICATOR);
                cx.local.right_leds.reset_pattern(MUTED_INDICATOR);

                Some(Monitor)
            }
            (Muted { mut high }, Animate(_)) => {
                high = !high;

                if high {
                    cx.local.left_leds.set_pattern(MUTED_INDICATOR);
                    cx.local.right_leds.set_pattern(MUTED_INDICATOR);
                } else {
                    cx.local.left_leds.reset_pattern(MUTED_INDICATOR);
                    cx.local.right_leds.reset_pattern(MUTED_INDICATOR);
                }

                Some(Muted { high })
            }
            _ => None,
        }
    }
}