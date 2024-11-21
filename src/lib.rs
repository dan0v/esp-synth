#![no_std]
#![no_main]
#![feature(const_fn_floating_point_arithmetic)]

extern crate alloc;

pub mod discrete_functions;
pub mod envelope;
pub mod filters;
pub mod i2s;
pub mod oscillators;
pub mod voice;
pub mod midi;
pub mod input;
