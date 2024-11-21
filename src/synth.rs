use crate::{
    envelope::{ADSREnvelope, Envelope},
    filters::{traits::Filter, BiquadHighPassFilter, BiquadLowPassFilter},
    oscillators::{
        scales::{freq, notes},
        traits::Oscillator,
        *,
    },
};
use esp_println::println;
#[allow(unused_imports)]
use helpers::{linear_map, log_map};
use midi_msg::{Channel, ChannelVoiceMsg, ControlChange, MidiMsg};

use alloc::{boxed::Box, vec, vec::Vec};

pub struct Voice {
    osc: Vec<Box<dyn Oscillator<Out = f32>>>,
    env: ADSREnvelope,
    lp: BiquadLowPassFilter,
    hp: BiquadHighPassFilter,
    note: Option<u8>,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            osc: vec![
                Box::new(SawToothOscillator::new(freq(notes::A4))),
                Box::new(SawToothOscillator::new(freq(notes::A4))),
                Box::new(SawToothOscillator::new(freq(notes::A3))),
                Box::new(SawToothOscillator::new(freq(notes::A3))),
                Box::new(Noise::new(0xBAD_5EED)),
            ],
            env: ADSREnvelope::new(0.01, 0.01, 0.6, 0.2),
            lp: BiquadLowPassFilter::new(),
            hp: BiquadHighPassFilter::new(),
            note: None,
        }
    }

    pub fn generate(&mut self) -> f32 {
        let osc_output = self.osc.iter_mut().map(|o| o.generate()).sum::<f32>();
        // let env_output = self.env.filter(osc_output);
        let lp_output = self.lp.filter(osc_output);
        lp_output
        // let hp_output = self.hp.filter(env_output);
        // // println!("{} {} {} {}", osc_output, env_output, lp_output, hp_output);
        //
        // 0.5 * (lp_output + hp_output)
    }

    pub fn handle_midi(&mut self, msg: MidiMsg) {
        if let MidiMsg::ChannelVoice {
            channel: Channel::Ch1,
            msg,
        } = msg
        {
            match msg {
                ChannelVoiceMsg::NoteOn { note, velocity } => {
                    self.handle_note_on(note, velocity);
                }
                ChannelVoiceMsg::NoteOff { note, velocity } => {
                    self.handle_note_off(note, velocity);
                }
                ChannelVoiceMsg::ControlChange { control } => {
                    self.handle_control_change(control);
                }
                _ => {}
            }
        }
    }

    fn handle_note_on(&mut self, note: u8, velocity: u8) {
        println!("on {}", note);
        self.note = Some(note);
        self.osc.iter_mut().for_each(|o| o.set_note(note));
        self.env.note_on(note, velocity)
    }

    fn handle_note_off(&mut self, note: u8, velocity: u8) {
        println!("off {}", note);
        self.note = None;
        self.env.note_off(note, velocity);
    }

    fn handle_control_change(&mut self, cc: ControlChange) {
        match cc {
            // Oscillators
            ControlChange::CC { control: 14, value } => {
                let tuning_factor = log_map(value, 1.1, 0., 5.);
                for i in (0..self.osc.len() - 1).step_by(2) {
                    self.osc[i].tune(tuning_factor);
                    self.osc[i + 1].tune(1. / tuning_factor);
                }
            }
            // Low-pass filter
            ControlChange::CC { control: 15, value } => {
                let cutoff_freq = log_map(value, 2.0, 7., 14.);
                self.lp.set_cutoff(cutoff_freq);
            }
            ControlChange::CC { control: 16, value } => {
                let q = log_map(value, 2., -4., 2.);
                self.lp.set_q(q);
            }
            // High-pass filter
            ControlChange::CC { control: 17, value } => {
                let cutoff_freq = log_map(value, 2.0, 7., 14.);
                self.hp.set_cutoff(cutoff_freq);
            }
            ControlChange::CC { control: 18, value } => {
                let q = log_map(value, 2., -4., 2.);
                self.hp.set_q(q);
            }
            // Envelope
            ControlChange::CC { control: 19, value } => {
                self.env.attack_time = log_map(value, 10., -4., 0.);
            }
            ControlChange::CC { control: 20, value } => {
                self.env.decay_time = log_map(value, 10., -4., 0.);
            }
            ControlChange::CC { control: 21, value } => {
                self.env.sustain_level = log_map(value, 10., -4., 0.);
            }
            ControlChange::CC { control: 22, value } => {
                self.env.release_time = log_map(value, 10., -4., 0.);
            }
            ControlChange::CC { control: 23, value } => {
                let delta_t = log_map(value, 10., -4., 0.);
                self.env.attack_time = delta_t;
                self.env.decay_time = delta_t;
                self.env.release_time = delta_t;
            }

            _ => {}
        }
    }
}

mod helpers {
    use micromath::F32Ext;

    /// Linearly maps a value from the range [0, 127] to the range [vmin, vmax].
    pub fn linear_map(value: u8, vmin: f32, vmax: f32) -> f32 {
        vmin + (vmax - vmin) * (value as f32 / 127.0)
    }

    /// Logarithmically maps a value from the range [0, 127] to the range [vmin, vmax].
    pub fn log_map(value: u8, base: f32, expmin: f32, expmax: f32) -> f32 {
        let x = linear_map(value, expmin, expmax);
        base.powf(x)
    }
}
