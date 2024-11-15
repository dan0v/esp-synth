use core::f32::consts::TAU;

use crate::i2s::SAMPLE_RATE;

use super::scales::REFERENCE_FREQ;

pub const DT: f32 = 1. / SAMPLE_RATE as f32;
/// calculate the phase increment for the set frequency, reference frequency
/// and tune of the oscillator
fn phase_increment(f_set: f32, f_ref: f32, tune: f32) -> f32 {
    TAU * f_set * DT * (f_ref / REFERENCE_FREQ) * tune
}

/// The phase generator is the heart of every oscillator. It's purpose is to produce the current
/// phase value [0, 2 * pi) at every generation step.
pub struct PhaseGenerator {
    phi: f32,  // current phase
    dphi: f32, // phase increment

    /// set frequency, range = (0, inf)
    f_set: f32,
    /// reference frequency, range = (0, inf), default = scales::REFERENCE_FREQ
    f_ref: f32,
    /// tuning factor, range = (0, inf), default = 1.0
    tune: f32,
}

impl PhaseGenerator {
    /// Create a new `Phaser`
    ///
    /// `f_ref` reference frequency (default = `scales::REFERENCE_FREQ`)
    pub fn new(f_ref: f32) -> Self {
        PhaseGenerator {
            phi: 0.0,
            dphi: 0.0,
            f_set: REFERENCE_FREQ,
            f_ref,
            tune: 1.0,
        }
    }

    pub fn tune(&mut self, tune: f32) {
        self.tune = tune;
        self.dphi = phase_increment(self.f_set, self.f_ref, self.tune);
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.f_set = frequency;
        self.dphi = phase_increment(self.f_set, self.f_ref, self.tune);
    }

    pub fn reset(&mut self) {
        self.phi = 0.;
    }

    pub fn generate(&mut self) -> f32 {
        let a = self.phi;
        self.phi += self.dphi;
        if self.phi > TAU {
            self.phi -= TAU;
        }
        a
    }
}

/// Helper trait that facilitates implementation of other traits
pub trait Phased {
    /// Return the underlying phase generator
    fn get_phase_generator(&mut self) -> &mut PhaseGenerator;
}
