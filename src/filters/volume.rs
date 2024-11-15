use core::iter::zip;

use super::traits::Filter;

pub struct Amplifier {
    gain: f32,
}

impl Amplifier {
    pub fn new() -> Self {
        Self { gain: 0.0 }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain
    }
}

impl Filter for Amplifier {
    type In = f32;
    type Out = f32;
    fn filter(&mut self, x: f32) -> f32 {
        (x * self.gain).clamp(-0.99, 0.99)
    }
}

pub struct Mixer<const N: usize> {
    levels: [f32; N],
}

impl<const N: usize> Mixer<N> {
    pub fn new() -> Self {
        Self {
            levels: [1. / N as f32; N],
        }
    }
}

impl<const N: usize> Filter for Mixer<N> {
    type In = [f32; N];
    type Out = f32;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        zip(self.levels, x).map(|(a, b)| a * b).sum()
    }
}
