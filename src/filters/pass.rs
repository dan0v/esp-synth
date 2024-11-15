use super::traits::Filter;
use crate::{
    discrete_functions::{cos, sin},
    oscillators::{phaser::DT, scales::REFERENCE_FREQ},
};
use core::f32::consts::TAU;

// https://www.dsprelated.com/freebooks/filters/Biquad_Software_Implementations.html
pub struct BiquadLowPassFilter {
    cutoff_freq: f32,
    q: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl BiquadLowPassFilter {
    pub fn new() -> Self {
        let mut flt = BiquadLowPassFilter {
            cutoff_freq: REFERENCE_FREQ,
            q: 0.72,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        };
        flt.update_coefficients();
        flt
    }

    pub fn set_cutoff(&mut self, cutoff_freq: f32) {
        self.cutoff_freq = cutoff_freq;
        self.update_coefficients();
    }

    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let omega = TAU * self.cutoff_freq * DT;
        let alpha = sin(omega) / (2.0 * self.q);
        let omega_cos = cos(omega);

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega_cos;
        let a2 = 1.0 - alpha;

        let b1 = 1.0 - omega_cos;
        let b0 = 0.5 * b1;
        let b2 = b0;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl Filter for BiquadLowPassFilter {
    type In = f32;
    type Out = f32;
    fn filter(&mut self, x: f32) -> f32 {
        let y = self.b0 * x - self.a1 * self.z1 - self.a2 * self.z2;
        self.z2 = self.z1;
        self.z1 = y;

        y + self.b1 * self.z1 + self.b2 * self.z2
    }
}

pub struct SimpleLowPassFilter {
    z1: f32,
    a: f32,
    b: f32,
}

impl SimpleLowPassFilter {
    pub fn new(alpha: f32) -> Self {
        SimpleLowPassFilter {
            z1: 0.0,
            a: 1.0 - alpha,
            b: alpha,
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        // let omega = TAU * cutoff_freq * DT;
        // let alpha = sin(omega);

        self.b = alpha;
        self.a = 1. - alpha;
    }
}

impl Filter for SimpleLowPassFilter {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: f32) -> f32 {
        let y = self.b * x - self.a * self.z1;
        self.z1 = y;

        y
    }
}
