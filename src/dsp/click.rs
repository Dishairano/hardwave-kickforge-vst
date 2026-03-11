//! Click / transient generator for the kick attack.
//!
//! Generates a short burst (noise, sine, or layered punch) that decays quickly,
//! producing the sharp transient "click" at the start of a hardstyle kick.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ClickType {
    /// Filtered white noise burst
    #[name = "Noise"]
    Noise,
    /// Short sine burst at click pitch
    #[name = "Sine"]
    Sine,
    /// Layered sine + noise for maximum impact
    #[name = "Punch"]
    Punch,
}

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
    /// Click frequency for sine-based clicks.
    click_freq: f32,
    /// Sine oscillator phase for sine-based clicks.
    sine_phase: f32,
    /// Click type selector.
    click_type: ClickType,
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
            click_freq: 4000.0,
            sine_phase: 0.0,
            click_type: ClickType::Noise,
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

    /// Set the click pitch (filter cutoff / sine frequency in Hz).
    pub fn set_pitch(&mut self, freq: f32) {
        self.click_freq = freq;
        self.filter_coeff = Self::calc_filter_coeff(self.sample_rate, freq);
    }

    /// Set the click generation type.
    pub fn set_click_type(&mut self, ct: ClickType) {
        self.click_type = ct;
    }

    /// Set the filter cutoff frequency for click shaping.
    pub fn set_filter_freq(&mut self, freq: f32) {
        self.filter_coeff = Self::calc_filter_coeff(self.sample_rate, freq);
    }

    fn calc_decay_coeff(sample_rate: f32, decay_ms: f32) -> f32 {
        let decay_samples = sample_rate * decay_ms / 1000.0;
        if decay_samples > 0.0 {
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
        self.sine_phase = 0.0;
        self.active = true;
    }

    /// Generate one sample of the click transient.
    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let raw = match self.click_type {
            ClickType::Noise => {
                // White noise through one-pole LP filter
                let noise = self.next_noise();
                self.filter_state += self.filter_coeff * (noise - self.filter_state);
                self.filter_state
            }
            ClickType::Sine => {
                // Short sine burst at click frequency
                let s = (self.sine_phase * std::f32::consts::TAU).sin();
                self.sine_phase += self.click_freq / self.sample_rate;
                self.sine_phase -= self.sine_phase.floor();
                s
            }
            ClickType::Punch => {
                // Layered: sine + noise for maximum transient impact
                let noise = self.next_noise();
                self.filter_state += self.filter_coeff * (noise - self.filter_state);
                let sine = (self.sine_phase * std::f32::consts::TAU).sin();
                self.sine_phase += self.click_freq / self.sample_rate;
                self.sine_phase -= self.sine_phase.floor();
                // Mix: 60% sine, 40% noise
                sine * 0.6 + self.filter_state * 0.4
            }
        };

        let sample = raw * self.level;

        // Apply envelope decay
        self.level *= self.decay_coeff;
        if self.level < 0.0001 {
            self.level = 0.0;
            self.active = false;
        }

        sample
    }

    /// Generate white noise via xorshift32 PRNG.
    #[inline]
    fn next_noise(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as i32 as f32) / (i32::MAX as f32)
    }

    pub fn reset(&mut self) {
        self.level = 0.0;
        self.filter_state = 0.0;
        self.sine_phase = 0.0;
        self.active = false;
    }
}
