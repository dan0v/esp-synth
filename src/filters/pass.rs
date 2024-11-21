use super::traits::Filter;
use crate::{
    discrete_functions::{cos, sin},
    oscillators::{phaser::DT, scales::REFERENCE_FREQ},
};
use core::f32::consts::TAU;

/// Generic filter in Direct Form II
///
/// Generic parameter is Order + 1
#[derive(Debug)]
pub struct DF2Filter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl DF2Filter {
    pub fn new(a: [f32; 3], b: [f32; 3]) -> Self {
        DF2Filter {
            b0: b[0] / a[0],
            b1: b[1] / a[0],
            b2: b[2] / a[0],
            a1: a[1] / a[0],
            a2: a[2] / a[0],
            z1: 0.,
            z2: 0.,
        }
    }

    pub fn set_coefficients(&mut self, a: [f32; 3], b: [f32; 3]) {
        self.b0 = b[0] / a[0];
        self.b1 = b[1] / a[0];
        self.b2 = b[2] / a[0];
        self.a1 = a[1] / a[0];
        self.a2 = a[2] / a[0];
        self.z1 = 0.;
        self.z2 = 0.;
    }
}

impl Filter for DF2Filter {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        let y = self.b0 * x - self.a1 * self.z1 - self.a2 * self.z2;
        self.z2 = self.z1;
        self.z1 = y;

        y + self.b1 * self.z1 + self.b2 * self.z2
    }
}

// https://pytorch.org/audio/main/_modules/torchaudio/functional/filtering.html#lowpass_biquad
pub struct BiquadLowPassFilter {
    df2: DF2Filter,
    cutoff_freq: f32,
    q: f32,
}

impl BiquadLowPassFilter {
    pub fn new() -> Self {
        let cutoff_freq = REFERENCE_FREQ;
        let q = 0.72;
        let (a, b) = Self::get_coefficients(cutoff_freq, q);
        BiquadLowPassFilter {
            df2: DF2Filter::new(a, b),
            cutoff_freq,
            q,
        }
    }

    pub fn set_cutoff(&mut self, cutoff_freq: f32) {
        self.cutoff_freq = cutoff_freq;
        let (a, b) = Self::get_coefficients(self.cutoff_freq, self.q);
        self.df2.set_coefficients(a, b);
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        let (a, b) = Self::get_coefficients(self.cutoff_freq, self.q);
        self.df2.set_coefficients(a, b);
    }

    fn get_coefficients(cutoff_freq: f32, q: f32) -> ([f32; 3], [f32; 3]) {
        let omega = TAU * cutoff_freq * DT;
        let alpha = sin(omega) / (2.0 * q);
        let omega_cos = cos(omega);

        (
            [1.0 + alpha, -2.0 * omega_cos, 1.0 - alpha],
            [
                0.5 - 0.5 * omega_cos,
                1.0 - omega_cos,
                0.5 - 0.5 * omega_cos,
            ],
        )
    }
}

impl Filter for BiquadLowPassFilter {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        self.df2.filter(x)
    }
}

// https://pytorch.org/audio/main/_modules/torchaudio/functional/filtering.html#highpass_biquad
pub struct BiquadHighPassFilter {
    df2: DF2Filter,
    cutoff_freq: f32,
    q: f32,
}

impl BiquadHighPassFilter {
    pub fn new() -> Self {
        let cutoff_freq = REFERENCE_FREQ;
        let q = 0.72;
        let (a, b) = Self::get_coefficients(cutoff_freq, q);
        BiquadHighPassFilter {
            df2: DF2Filter::new(a, b),
            cutoff_freq,
            q,
        }
    }

    pub fn set_cutoff(&mut self, cutoff_freq: f32) {
        self.cutoff_freq = cutoff_freq;
        let (a, b) = Self::get_coefficients(self.cutoff_freq, self.q);
        self.df2.set_coefficients(a, b);
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        let (a, b) = Self::get_coefficients(self.cutoff_freq, self.q);
        self.df2.set_coefficients(a, b);
    }

    fn get_coefficients(cutoff_freq: f32, q: f32) -> ([f32; 3], [f32; 3]) {
        let omega = TAU * cutoff_freq * DT;
        let alpha = sin(omega) / (2.0 * q);
        let omega_cos = cos(omega);

        (
            [1.0 + alpha, -2.0 * omega_cos, 1.0 - alpha],
            [
                0.5 + 0.5 * omega_cos,
                -1.0 - omega_cos,
                0.5 + 0.5 * omega_cos,
            ],
        )
    }
}

impl Filter for BiquadHighPassFilter {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        self.df2.filter(x)
    }
}
