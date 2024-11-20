use super::traits::Filter;
use crate::{
    discrete_functions::{cos, sin},
    oscillators::{phaser::DT, scales::REFERENCE_FREQ},
};
use core::f32::consts::TAU;

/// Generic filter in Direct Form II
///
/// Generic parameter is Order + 1
pub struct DF2Filter<const N: usize> {
    a: [f32; N],
    b: [f32; N],
    v: [f32; N],
}

impl<const N: usize> DF2Filter<N> {
    pub const ORDER: usize = N - 1;

    pub fn new(a: [f32; N], b: [f32; N]) -> Self {
        DF2Filter { a, b, v: [0.; N] }
    }

    pub fn set_coefficients(&mut self, a: [f32; N], b: [f32; N]) {
        self.b = b.map(|x| x / a[0]);
        self.a = a.map(|x| x / a[0]);
    }
}

impl<const N: usize> Filter for DF2Filter<N> {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        let mut v0 = self.b[0] * x;
        for i in 1..N {
            v0 -= self.a[i] * self.v[i];
            self.v[i] = self.v[i - 1];
        }
        self.v[0] = v0;

        let mut y = v0;
        for i in 1..N {
            y += self.b[i] * self.v[i];
        }
        y
    }
}

// https://pytorch.org/audio/main/_modules/torchaudio/functional/filtering.html#lowpass_biquad
pub struct BiquadLowPassFilter {
    df2: DF2Filter<3>,
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
    df2: DF2Filter<3>,
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

