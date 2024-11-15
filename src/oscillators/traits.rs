use super::{
    phaser::{PhaseGenerator, Phased},
    scales::freq,
};

pub trait Generator {
    /// The output type of the generator
    type Out;

    /// Produce the next output value
    ///
    /// Should only be called once per time step
    fn generate(&mut self) -> Self::Out;
}

pub trait Oscillator: Generator {
    /// Tune the oscillator
    ///
    /// A tuning factor of 1.0 means the oscillator is perfectly in tune
    /// and produces exactly the frequency to which it is set.
    fn tune(&mut self, tuning_factor: f32);

    /// Set the nominal frequency of the oscillator
    ///
    /// The produced frequency `f_out` is affected by the tuning factor `c` and the reference
    /// frequency `f_ref`: f_out = f_ref * f_set * c
    fn set_frequency(&mut self, frequency: f32);

    /// Set the nominal frequency by note
    fn set_note(&mut self, note: u8);

    /// Reset the internal state (e.g. phase) to initial values
    fn reset(&mut self);
}

// blanket implementation for phased generators, i.e. generators whose output depends on an
// internal phase
impl<T: Phased + Generator> Oscillator for T {
    fn tune(&mut self, tuning_factor: f32) {
        self.get_phase_generator().tune(tuning_factor);
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.get_phase_generator().set_frequency(frequency);
    }

    fn set_note(&mut self, note: u8) {
        self.set_frequency(freq(note));
    }

    fn reset(&mut self) {
        self.get_phase_generator().reset();
    }
}

// // Arrays ==============================================================
// // An array of oscillators will produce an array of signals
//
// impl<const N: usize, O: Generator> Generator for [O; N] {
//     type Out = [O::Out; N];
//
//     fn reset(&mut self) {
//         for gen in self {
//             gen.reset()
//         }
//     }
//
//     fn generate(&mut self) -> Self::Out {
//         let mut out: Self::Out = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
//         for i in 0..N {
//             out[i] = self[i].generate()
//         }
//         out
//     }
// }
//
// impl<const N: usize, O: Oscillator> Oscillator for [O; N] {
//     fn tune(&mut self, tuning_factor: f32) {
//         self.iter_mut().for_each(|osc| osc.tune(tuning_factor));
//     }
//
//     fn set_frequency(&mut self, frequency: f32) {
//         self.iter_mut().for_each(|osc| osc.set_frequency(frequency));
//     }
// }
//
// // Tuples ==============================================================
// // A tuple of N generators, where all generators have the same output type,
// // produces an array of N signals
//
// macro_rules! count {
//     () => { 0 };
//     ($head:tt$(, $tail:tt)*) => { 1 + count!($($tail),*) };
// }
//
// macro_rules! impl_generator_for_tuple {
//     ($first:ident$(, $name:ident)*) => {
//         impl<$first, $($name, )*> Generator for ($first, $($name, )*)
//         where
//             $first: Generator,
//             $($name: Generator<Out = $first::Out>,)*
//         {
//             type Out = [$first::Out; count!($first$(, $name)*)];
//             #[allow(non_snake_case)]
//             fn generate(&mut self) -> Self::Out {
//                 let ($first, $($name,)*) = self;
//                 [$first.generate() $(, $name.generate())*]
//             }
//         }
//     };
// }
//
// impl_generator_for_tuple!(A);
// impl_generator_for_tuple!(A, B);
// impl_generator_for_tuple!(A, B, C);
// impl_generator_for_tuple!(A, B, C, D);
// impl_generator_for_tuple!(A, B, C, D, E);
// impl_generator_for_tuple!(A, B, C, D, E, F);
//
// macro_rules! impl_oscillator_for_tuple {
//     ($first:ident$(, $name:ident)*) => {
//         impl<$first, $($name, )*> Oscillator for ($first, $($name, )*)
//         where
//             $first: Oscillator,
//             $($name: Oscillator<Out = $first::Out>,)*
//         {
//             #[allow(non_snake_case)]
//             fn set_frequency(&mut self, frequency: f32) {
//                 let ($first, $($name,)*) = self;
//                 $first.set_frequency(frequency);
//                 $($name.set_frequency(frequency);)*
//             }
//
//             #[allow(non_snake_case)]
//             fn tune(&mut self, tuning_factor: f32) {
//                 let ($first, $($name,)*) = self;
//                 $first.tune(tuning_factor);
//                 $($name.tune(tuning_factor);)*
//             }
//         }
//     };
// }
//
// impl_oscillator_for_tuple!(A);
// impl_oscillator_for_tuple!(A, B);
// impl_oscillator_for_tuple!(A, B, C);
// impl_oscillator_for_tuple!(A, B, C, D);
// impl_oscillator_for_tuple!(A, B, C, D, E);
// impl_oscillator_for_tuple!(A, B, C, D, E, F);
