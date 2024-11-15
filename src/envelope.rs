use crate::{filters::traits::Filter, oscillators::phaser::DT};

pub trait Envelope: Filter {
    fn note_on(&mut self, note: u8, velocity: u8);
    fn note_off(&mut self, note: u8, velocity: u8);
}

#[derive(Debug)]
pub struct ADSREnvelope {
    pub attack_time: f32,
    pub decay_time: f32,
    pub sustain_level: f32,
    pub release_time: f32,
    stage: ADSRStage,
    level: f32,
}

#[derive(Debug)]
enum ADSRStage {
    Attack,
    DecaySustain,
    Release,
    Idle,
}

impl ADSREnvelope {
    pub fn new(attack_time: f32, decay_time: f32, sustain_level: f32, release_time: f32) -> Self {
        ADSREnvelope {
            attack_time,
            decay_time,
            sustain_level,
            release_time,
            stage: ADSRStage::Idle,
            level: 0.0,
        }
    }
}

impl Envelope for ADSREnvelope {
    fn note_on(&mut self, _: u8, _: u8) {
        self.stage = ADSRStage::Attack;
    }

    fn note_off(&mut self, _: u8, _: u8) {
        self.stage = ADSRStage::Release;
    }
}

impl Filter for ADSREnvelope {
    type In = f32;
    type Out = f32;

    fn filter(&mut self, x: f32) -> f32 {
        match self.stage {
            ADSRStage::Attack => {
                self.level += DT / self.attack_time;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = ADSRStage::DecaySustain;
                }
            }
            ADSRStage::DecaySustain => {
                self.level -= DT * (self.level - self.sustain_level) / self.decay_time;
            }
            ADSRStage::Release => {
                self.level -= DT * self.sustain_level / self.release_time;
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = ADSRStage::Idle;
                }
            }
            ADSRStage::Idle => {}
        }
        x * self.level
    }
}
