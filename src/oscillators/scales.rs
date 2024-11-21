pub const HALF_TONE_FACTOR: f32 = 1.05946309436; // 2^(1/12)
pub const REFERENCE_FREQ: f32 = 440.0;

pub mod notes {
    pub const A0: u8 = 1;
    pub const A1: u8 = 13;
    pub const A2: u8 = 25;
    pub const A3: u8 = 37;
    pub const A4: u8 = 49;
    pub const A5: u8 = 61;
    pub const A6: u8 = 73;
    pub const A7: u8 = 85;
}

const fn create_tempered_scale(pitch: f32) -> [f32; 88] {
    let mut scale = [0.0; 88];
    let i0 = notes::A4 as usize;
    scale[i0] = pitch;
    let mut i = i0;
    while i < scale.len() - 1 {
        scale[i + 1] = HALF_TONE_FACTOR * scale[i];
        i += 1;
    }
    i = i0;
    while i > 0 {
        scale[i - 1] = scale[i] / HALF_TONE_FACTOR;
        i -= 1;
    }

    scale
}

pub const TEMPERED_SCALE: [f32; 88] = create_tempered_scale(REFERENCE_FREQ);

pub fn freq(n: u8) -> f32 {
    TEMPERED_SCALE[n as usize]
}
