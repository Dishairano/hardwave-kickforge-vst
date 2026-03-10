//! Click / transient generator for the kick attack.
//!
//! Generates a short burst of filtered noise that decays quickly,
//! producing the sharp transient "click" at the start of a hardstyle kick.

pub struct Click {
    sample_rate: f32,
    /// Envelope level (decays from 1.0 toward 0.0).
    level: f32,
    /// Per-sample decay multiplier.
    decay_coeff: f32,
    /// Simple one-pole filter state for shaping the click tone.
    filter_state: f32,
    /// Filter coefficient derived from click pitch.
    filter_coeff: f32,
    /// PRNG state for noise generation (xorshift32).
    rng_state: u32,
    active: bool,
}

impl Click {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            level: 0.0,
            decay_coeff: Self::calc_decay_coeff(sample_rate, 5.0),
            filter_state: 0.0,
            filter_coeff: Self::calc_filter_coeff(sample_rate, 4000.0),
            rng_state: 0x12345678,
            active: false,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    /// Set decay time in milliseconds.
    pub fn set_decay_ms(&mut self, ms: f32) {
        self.decay_coeff = Self::calc_decay_coeff(self.sample_rate, ms);
    }

    /// Set the click pitch (high-pass filter cutoff in Hz).
    pub fn set_pitch(&mut self, freq: f32) {
        self.filter_coeff = Self::calc_filter_coeff(self.sample_rate, freq);
    }

    fn calc_decay_coeff(sample_rate: f32, decay_ms: f32) -> f32 {
        let decay_samples = sample_rate * decay_ms / 1000.0;
        if decay_samples > 0.0 {
            // Coefficient so that level reaches ~0.001 after decay_samples
            0.001_f32.powf(1.0 / decay_samples)
        } else {
            0.0
        }
    }

    fn calc_filter_coeff(sample_rate: f32, freq: f32) -> f32 {
        let fc = freq.min(sample_rate * 0.49);
        let rc = 1.0 / (std::f32::consts::TAU * fc);
        let dt = 1.0 / sample_rate;
        dt / (rc + dt)
    }

    /// Trigger the click (call on note-on).
    pub fn trigger(&mut self) {
        self.level = 1.0;
        self.filter_state = 0.0;
        self.active = true;
    }

    /// Generate one sample of the click transient.
    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Generate white noise via xorshift32
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        let noise = (self.rng_state as i32 as f32) / (i32::MAX as f32);

        // One-pole low-pass filter to shape the noise
        self.filter_state += self.filter_coeff * (noise - self.filter_state);

        let sample = self.filter_state * self.level;

        // Apply envelope decay
        self.level *= self.decay_coeff;
        if self.level < 0.0001 {
            self.level = 0.0;
            self.active = false;
        }

        sample
    }

    pub fn reset(&mut self) {
        self.level = 0.0;
        self.filter_state = 0.0;
        self.active = false;
    }
}
