use core::f32::consts::{FRAC_PI_2, TAU};
use micromath::F32Ext;

const TABLE_SIZE: usize = 64;
const DPHI: f32 = TAU / TABLE_SIZE as f32;

const fn factorial(n: u32) -> u32 {
    let mut f = 1;
    let mut i = 0;
    while i < n {
        i += 1;
        f *= i;
    }
    f
}

const fn pow(x: f32, n: u32) -> f32 {
    let mut p = 1.;
    let mut i = 0;
    while i < n {
        p *= x;
        i += 1;
    }
    p
}

const fn sin_apx(phi: f32, n: u32) -> f32 {
    let mut result = 0.;
    let mut i = 1;
    let mut s = 1.;
    while i < n {
        result += s * pow(phi, i) / factorial(i) as f32;
        i += 2;
        s *= -1.;
    }
    result
}

const fn generate_sin_table() -> [f32; TABLE_SIZE] {
    let mut table = [0.; TABLE_SIZE];
    let n = TABLE_SIZE / 4;

    table[0] = 0.;
    table[n] = 1.;
    table[2 * n] = 0.;
    table[3 * n] = -1.;

    let mut phi = DPHI;
    let mut i = 1;
    while i < n {
        table[i] = sin_apx(phi, 10);
        table[2 * n - i] = table[i];
        table[2 * n + i] = -table[i];
        table[4 * n - i] = -table[i];
        phi += DPHI;
        i += 1;
    }

    table
}

const SIN: [f32; TABLE_SIZE] = generate_sin_table();

pub fn sin(phi: f32) -> f32 {
    if phi > TAU {
        sin(phi - TAU)
    } else if phi < 0. {
        -sin(-phi)
    } else {
        let mut d = phi / DPHI;
        let i: usize = d.floor() as usize;
        d -= i as f32;

        let a = SIN[i];
        let b = SIN[(i + 1) % TABLE_SIZE];
        d * b + (1. - d) * a
    }
}

pub fn cos(phi: f32) -> f32 {
    sin(phi + FRAC_PI_2)
}
