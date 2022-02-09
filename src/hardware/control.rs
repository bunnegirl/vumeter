use crate::runtime::{State, State::*};
#[allow(unused_imports)]
use rtt_target::*;
use stm32f4xx_hal::gpio::*;

#[derive(Clone, Copy, Debug)]
pub enum AudioOutput {
    Headphones,
    Speakers,
}

pub type AudioOutputDsp = Pin<Output<PushPull>, 'B', 12>;
pub type AudioOutputCtrl = Pin<Output<PushPull>, 'B', 13>;
pub type AudioMuteCtrl = Pin<Output<PushPull>, 'B', 14>;

pub struct Control {
    audio_output_dsp: AudioOutputDsp,
    audio_output_ctrl: AudioOutputCtrl,
    audio_mute_ctrl: AudioMuteCtrl,
}

impl Control {
    pub fn new(
        audio_output_dsp: AudioOutputDsp,
        audio_output_ctrl: AudioOutputCtrl,
        audio_mute_ctrl: AudioMuteCtrl,
    ) -> Self {
        Self {
            audio_output_dsp,
            audio_output_ctrl,
            audio_mute_ctrl,
        }
    }

    pub fn read(&mut self) {}

    pub fn write(&mut self, state: &State) {
        use AudioOutput::*;

        if let Running { audio_output, .. } = state {
            match audio_output {
                Headphones => {
                    self.audio_output_dsp.set_low();
                    self.audio_output_ctrl.set_low();
                }
                Speakers => {
                    self.audio_output_dsp.set_high();
                    self.audio_output_ctrl.set_high();
                }
            }
        }

        if let Running { audio_mute, .. } = state {
            if *audio_mute {
                self.audio_mute_ctrl.set_high();
            } else {
                self.audio_mute_ctrl.set_low();
            }
        }
    }

    pub fn clock(&mut self) {
        //
    }
}
