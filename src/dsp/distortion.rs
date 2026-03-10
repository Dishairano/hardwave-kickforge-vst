//! Tanh-based waveshaper distortion for kick body drive.
//!
//! `drive` controls how much the signal is pushed into the tanh curve.
//! At drive=0 the signal passes through clean; at drive=1 it's heavily
//! saturated.

pub struct Distortion {
    /// Drive amount 0.0..1.0, mapped to a gain multiplier internally.
    drive: f32,
}

impl Distortion {
    pub fn new() -> Self {
        Self { drive: 0.0 }
    }

    /// Set drive amount (0.0 = clean, 1.0 = heavy saturation).
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Process a single sample through the waveshaper.
    pub fn process(&self, sample: f32) -> f32 {
        if self.drive < 0.001 {
            return sample;
        }
        // Map drive 0..1 to gain 1..20
        let gain = 1.0 + self.drive * 19.0;
        let driven = (sample * gain).tanh();
        // Compensate for volume increase from saturation
        let compensation = 1.0 / (gain * 0.15 + 0.85).min(1.5);
        driven * compensation
    }

    pub fn reset(&mut self) {
        // Stateless — nothing to reset
    }
}
