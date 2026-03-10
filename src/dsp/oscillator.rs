//! Simple sine oscillator for kick synthesis.
//!
//! Maintains phase continuity across frequency changes (important for
//! smooth pitch sweeps).

pub struct Oscillator {
    sample_rate: f32,
    phase: f32,
    frequency: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 50.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    /// Generate one sample of a sine wave at the current frequency.
    pub fn process(&mut self) -> f32 {
        let sample = (self.phase * std::f32::consts::TAU).sin();

        self.phase += self.frequency / self.sample_rate;
        // Keep phase in [0, 1) to avoid floating-point precision loss over time
        self.phase -= self.phase.floor();

        sample
    }

    /// Reset the oscillator phase (call on note-on for consistent attack).
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}
