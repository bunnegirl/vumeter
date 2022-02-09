use crate::hardware::keypad::Key;
use crate::hardware::meter::MeterChannel;
use crate::hardware::time;
use fugit::ExtU32;
use heapless::mpmc::Q8;

pub use Message::*;
pub use State::*;

pub const DB_PLUS_12: f32 = 0.9800;
// pub const DB_PLUS_9: f32 = 0.9500;
pub const DB_PLUS_6: f32 = 0.9200;
pub const DB_PLUS_3: f32 = 0.8900;
pub const DB_NOMINAL: f32 = 0.8600;
pub const DB_MINUS_3: f32 = 0.8300;
pub const DB_MINUS_6: f32 = 0.8000;
// pub const DB_MINUS_9: f32 = 0.7700;
pub const DB_MINUS_12: f32 = 0.7400;
// pub const DB_MINUS_15: f32 = 0.7100;
pub const DB_MINUS_18: f32 = 0.6800;
// pub const DB_MINUS_21: f32 = 0.6500;
// pub const DB_MINUS_24: f32 = 0.6200;
pub const DB_MINUS_27: f32 = 0.5900;
// pub const DB_MINUS_30: f32 = 0.5600;
// pub const DB_MINUS_33: f32 = 0.5300;
pub const DB_MINUS_36: f32 = 0.5000;
// pub const DB_MINUS_39: f32 = 0.4700;
// pub const DB_MINUS_42: f32 = 0.4400;
pub const DB_MINUS_45: f32 = 0.4100;
// pub const DB_MINUS_48: f32 = 0.3800;
// pub const DB_MINUS_51: f32 = 0.3500;
pub const DB_MINUS_54: f32 = 0.3200;
// pub const DB_MINUS_57: f32 = 0.2900;
// pub const DB_MINUS_60: f32 = 0.2600;
// pub const DB_MINUS_63: f32 = 0.2300;
// pub const DB_MINUS_66: f32 = 0.2000;
pub const DB_MINUS_INF: f32 = 0.0;

// Input, Peak Delay, Level Delay
pub const LEVELS: [(f32, u32, u32); 13] = [
    (DB_PLUS_12, 2400, 20),
    (DB_PLUS_6, 1500, 20),
    (DB_PLUS_3, 900, 20),
    (DB_NOMINAL, 600, 20),
    (DB_MINUS_3, 300, 20),
    (DB_MINUS_6, 300, 20),
    (DB_MINUS_12, 300, 20),
    (DB_MINUS_18, 300, 20),
    (DB_MINUS_27, 300, 20),
    (DB_MINUS_36, 300, 20),
    (DB_MINUS_45, 300, 20),
    (DB_MINUS_54, 300, 20),
    (DB_MINUS_INF, 300, 20),
];

pub static Q: Q8<Message> = Q8::new();

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Booted,
    KeypadUpdate(Key),
    MeterUpdate(f32, f32),
}

impl Message {
    pub fn send(self) {
        Q.enqueue(self).ok();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Booting,
    Running {
        left: MeterChannel,
        right: MeterChannel,
        peaks: bool,
        levels: bool,
        headphones: bool,
        speakers: bool,
    },
    Standby,
}

impl State {
    #[must_use]
    pub fn recv(mut self, msg: Message) -> State {
        match (&mut self, msg) {
            (Booting, Booted) => {
                return Running {
                    left: MeterChannel::default(),
                    right: MeterChannel::default(),
                    peaks: true,
                    levels: true,
                    headphones: true,
                    speakers: false,
                }
            }

            // calculate meter peak and level
            (Running { left, right, .. }, MeterUpdate(left_raw, right_raw)) => {
                let calculate = |channel: &mut MeterChannel, channel_raw: f32| {
                    let now = time::now();
                    let new = LEVELS
                        .iter()
                        .enumerate()
                        .find(|(_, (level, _, _))| channel_raw >= *level);

                    if let Some((index, (_, peak_decay_ms, level_decay_ms))) = new {
                        let new_level = 0b1111_1111_1111 >> index;
                        let new_peak = 0b1000_0000_0000 >> index;

                        if new_peak >= channel.peak || channel.peak_decay < now {
                            channel.peak = new_peak;
                            channel.peak_decay = time::now() + peak_decay_ms.millis();
                        }

                        if new_level >= channel.level || channel.level_decay < now {
                            channel.level = new_level;
                            channel.level_decay = time::now() + level_decay_ms.millis();
                        }
                    }
                };

                calculate(left, left_raw);
                calculate(right, right_raw);
            }

            // toggle meter peaks
            (Running { peaks, .. }, KeypadUpdate(Key::TogglePeaks)) => {
                *peaks = !*peaks;
            }

            // toggle meter levels
            (Running { levels, .. }, KeypadUpdate(Key::ToggleLevels)) => {
                *levels = !*levels;
            }

            // toggle headphones
            (Running { headphones, .. }, KeypadUpdate(Key::ToggleHeadphones)) => {
                *headphones = !*headphones;
            }

            // toggle speakers
            (Running { speakers, .. }, KeypadUpdate(Key::ToggleSpeakers)) => {
                *speakers = !*speakers;
            }

            _ => {}
        };

        self
    }
}
