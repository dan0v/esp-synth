use super::{
    phaser::{PhaseGenerator, Phased},
    traits::{Generator, Oscillator},
};
use crate::discrete_functions::sin;
use core::f32::consts::{PI, TAU};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

///  Implementation of various oscillators
///
///  When the type implements [`Phased`] and [`Generator`] it will automatically
///  implement [`Oscillator`] as there is blanket implementation.

// Sine =================================
//
//  1 ┤  -           -
//    ┤*   *       *   *       *
//    ┼─────*─────2π────*─────4π─>
//    ┤      *   *       *   *
// ─1 ┤        -           -

pub struct SineOscillator {
    phase_gen: PhaseGenerator,
}

impl SineOscillator {
    pub fn new(f_ref: f32) -> Self {
        Self {
            phase_gen: PhaseGenerator::new(f_ref),
        }
    }
}

impl Generator for SineOscillator {
    type Out = f32;

    fn generate(&mut self) -> f32 {
        let phi = self.phase_gen.generate();
        sin(phi)
    }
}

impl Phased for SineOscillator {
    fn get_phase_generator(&mut self) -> &mut PhaseGenerator {
        &mut self.phase_gen
    }
}

// Sawtooth =============================
//
//  1 ┤        /        /
//    ┤      / │      / │
//    ┼────/───2π───/───4π───/─>
//    ┤  /     │  /     │  /
// ─1 ┤/       │/       │/

pub struct SawToothOscillator {
    phase_gen: PhaseGenerator,
}

impl SawToothOscillator {
    pub fn new(f_ref: f32) -> Self {
        Self {
            phase_gen: PhaseGenerator::new(f_ref),
        }
    }
}

impl Generator for SawToothOscillator {
    type Out = f32;

    fn generate(&mut self) -> Self::Out {
        let phi = self.phase_gen.generate();
        phi / PI - 1.0
    }
}

impl Phased for SawToothOscillator {
    fn get_phase_generator(&mut self) -> &mut PhaseGenerator {
        &mut self.phase_gen
    }
}
// PWM ==================================
//
//  1 ┤───┐     ┌───┐     ┌
//    ┤   │     │   │     │
//    ┼───│─────2π──│─────4π─>
//    ┤   │     │   │     │
// ─1 ┤   └─────┘   └─────┘

pub struct PWMOscillator {
    phase_gen: PhaseGenerator,

    /// The duty cycle is the fraction of one period in which the oscillator produces a high
    /// signal.
    ///
    /// range:  (0, 1)
    pub duty_cycle: f32,
}

impl PWMOscillator {
    pub fn new(f_ref: f32, duty_cycle: f32) -> Self {
        Self {
            phase_gen: PhaseGenerator::new(f_ref),
            duty_cycle,
        }
    }
}

impl Generator for PWMOscillator {
    type Out = f32;

    fn generate(&mut self) -> f32 {
        let phi = self.phase_gen.generate();
        if phi < TAU * self.duty_cycle {
            1.0
        } else {
            -1.0
        }
    }
}

impl Phased for PWMOscillator {
    fn get_phase_generator(&mut self) -> &mut PhaseGenerator {
        &mut self.phase_gen
    }
}

// ======================================

pub struct Noise {
    rng: XorShiftRng,
}

impl Noise {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: XorShiftRng::from_seed(bytemuck::cast([seed, seed, seed, seed])),
        }
    }
}

impl Generator for Noise {
    type Out = f32;
    fn generate(&mut self) -> f32 {
        // generate a random i32
        let a: i32 = unsafe { core::mem::transmute(self.rng.next_u32()) };
        // map to [-1.0, 1.0]
        a as f32 / i32::MAX as f32
    }
}

// Do-nothing implementation of Oscillator for Noise
impl Oscillator for Noise {
    fn tune(&mut self, _tuning_factor: f32) {}
    fn set_frequency(&mut self, _frequency: f32) {}
    fn set_note(&mut self, _note: u8) {}
    fn reset(&mut self) {}
}
