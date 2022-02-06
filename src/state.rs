use crate::app::monotonics as time;
use crate::meter::MeterChannel;
use fugit::ExtU32;
use heapless::mpmc::Q8;

pub use Message::*;
pub use State::*;

pub const DB_PLUS_12: f32 = 0.9800;
pub const DB_PLUS_9: f32 = 0.9500;
pub const DB_PLUS_6: f32 = 0.9200;
pub const DB_PLUS_3: f32 = 0.8900;
pub const DB_NOMINAL: f32 = 0.8600;
pub const DB_MINUS_3: f32 = 0.8300;
pub const DB_MINUS_6: f32 = 0.8000;
pub const DB_MINUS_9: f32 = 0.7700;
pub const DB_MINUS_12: f32 = 0.7400;
pub const DB_MINUS_15: f32 = 0.7100;
pub const DB_MINUS_18: f32 = 0.6800;
pub const DB_MINUS_21: f32 = 0.6500;
pub const DB_MINUS_24: f32 = 0.6200;
pub const DB_MINUS_27: f32 = 0.5900;
pub const DB_MINUS_30: f32 = 0.5600;
pub const DB_MINUS_33: f32 = 0.5300;
pub const DB_MINUS_36: f32 = 0.5000;
pub const DB_MINUS_39: f32 = 0.4700;
pub const DB_MINUS_42: f32 = 0.4400;
pub const DB_MINUS_45: f32 = 0.4100;
pub const DB_MINUS_48: f32 = 0.3800;
pub const DB_MINUS_51: f32 = 0.3500;
pub const DB_MINUS_54: f32 = 0.3200;
pub const DB_MINUS_57: f32 = 0.2900;
pub const DB_MINUS_60: f32 = 0.2600;
pub const DB_MINUS_63: f32 = 0.2300;
pub const DB_MINUS_66: f32 = 0.2000;

// Input, Peak Delay, Level Delay
const LEVELS: [(f32, u32, u32); 12] = [
    (DB_PLUS_12, 2400, 50),
    (DB_PLUS_6, 1500, 50),
    (DB_PLUS_3, 900, 50),
    (DB_NOMINAL, 600, 50),
    (DB_MINUS_3, 300, 50),
    (DB_MINUS_6, 300, 50),
    (DB_MINUS_12, 300, 50),
    (DB_MINUS_18, 300, 50),
    (DB_MINUS_27, 300, 50),
    (DB_MINUS_36, 300, 50),
    (DB_MINUS_45, 300, 50),
    (DB_MINUS_54, 300, 50),
];

pub static Q: Q8<Message> = Q8::new();

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Initialise,
    KeypadUpdate(usize),
    MeterUpdate(f32, f32),
    MeterDecay,
}

impl Message {
    pub fn send(self) {
        Q.enqueue(self).ok();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShowPeaks {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub enum ShowLevels {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub enum HeadphoneOutput {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub enum SpeakerOutput {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Uninitialised,
    VolumeMeter(
        MeterChannel,
        MeterChannel,
        ShowPeaks,
        ShowLevels,
        HeadphoneOutput,
        SpeakerOutput,
    ),
}

impl State {
    pub fn with_show_peaks(self, toggle: ShowPeaks) -> State {
        if let VolumeMeter(left, right, _, levels, headphones, speakers) = self {
            VolumeMeter(left, right, toggle, levels, headphones, speakers)
        } else {
            self
        }
    }

    pub fn with_show_levels(self, toggle: ShowLevels) -> State {
        if let VolumeMeter(left, right, peaks, _, headphones, speakers) = self {
            VolumeMeter(left, right, peaks, toggle, headphones, speakers)
        } else {
            self
        }
    }

    pub fn with_headphone_output(self, toggle: HeadphoneOutput) -> State {
        if let VolumeMeter(left, right, peaks, levels, _, speakers) = self {
            VolumeMeter(left, right, peaks, levels, toggle, speakers)
        } else {
            self
        }
    }

    pub fn with_speaker_output(self, toggle: SpeakerOutput) -> State {
        if let VolumeMeter(left, right, peaks, levels, headphones, _) = self {
            VolumeMeter(left, right, peaks, levels, headphones, toggle)
        } else {
            self
        }
    }

    pub fn recv(self, msg: Message) -> State {
        match (self, msg) {
            (Uninitialised, Initialise) => VolumeMeter(
                MeterChannel::default(),
                MeterChannel::default(),
                ShowPeaks::On,
                ShowLevels::On,
                HeadphoneOutput::On,
                SpeakerOutput::Off,
            ),

            (VolumeMeter(..), MeterUpdate(left_raw, right_raw)) => {
                meter_update(self, left_raw, right_raw)
            }

            (VolumeMeter(..), MeterDecay) => meter_decay(self),

            // toggle meter peaks
            (VolumeMeter(_, _, ShowPeaks::Off, ..), KeypadUpdate(0)) => {
                self.with_show_peaks(ShowPeaks::On)
            }
            (VolumeMeter(_, _, ShowPeaks::On, ..), KeypadUpdate(0)) => {
                self.with_show_peaks(ShowPeaks::Off)
            }

            // toggle meter levels
            (VolumeMeter(_, _, _, ShowLevels::Off, ..), KeypadUpdate(1)) => {
                self.with_show_levels(ShowLevels::On)
            }
            (VolumeMeter(_, _, _, ShowLevels::On, ..), KeypadUpdate(1)) => {
                self.with_show_levels(ShowLevels::Off)
            }

            // toggle output mute
            (VolumeMeter(_, _, _, _, HeadphoneOutput::Off, ..), KeypadUpdate(2)) => {
                self.with_headphone_output(HeadphoneOutput::On)
            }
            (VolumeMeter(_, _, _, _, HeadphoneOutput::On, ..), KeypadUpdate(2)) => {
                self.with_headphone_output(HeadphoneOutput::Off)
            }

            // toggle speaker output
            (VolumeMeter(_, _, _, _, _, SpeakerOutput::Off), KeypadUpdate(3)) => {
                self.with_speaker_output(SpeakerOutput::On)
            }
            (VolumeMeter(_, _, _, _, _, SpeakerOutput::On), KeypadUpdate(3)) => {
                self.with_speaker_output(SpeakerOutput::Off)
            }

            _ => self,
        }
    }
}

pub fn meter_update(state: State, left_raw: f32, right_raw: f32) -> State {
    let calculate = |channel: &mut MeterChannel, input: f32| {
        let new = LEVELS
            .iter()
            .enumerate()
            .find(|(_, (level, _, _))| input >= *level);

        if let Some((index, (_, peak_decay_ms, level_decay_ms))) = new {
            let new_level = 0b1111_1111_1111 >> index;
            let new_peak = 0b1000_0000_0000 >> index;

            if new_peak >= channel.peak {
                channel.peak = new_peak;
                channel.peak_decay = time::now() + peak_decay_ms.millis();
            }

            if new_level >= channel.level {
                channel.level = new_level;
                channel.level_decay = time::now() + level_decay_ms.millis();
            }
        }
    };

    if let VolumeMeter(mut left, mut right, peaks, levels, headphones, speakers) = state {
        calculate(&mut left, left_raw);
        calculate(&mut right, right_raw);

        VolumeMeter(left, right, peaks, levels, headphones, speakers)
    } else {
        state
    }
}

fn meter_decay(state: State) -> State {
    if let VolumeMeter(mut left, mut right, peaks, levels, headphones, speakers) = state {
        let now = time::now();

        if left.level_decay < now {
            left.level = 0;
        }

        if right.level_decay < now {
            right.level = 0;
        }

        if left.peak_decay < now {
            left.peak = 0;
        }

        if right.peak_decay < now {
            right.peak = 0;
        }

        VolumeMeter(left, right, peaks, levels, headphones, speakers)
    } else {
        state
    }
}
