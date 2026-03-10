//! State-variable filter (SVF) for kick tone shaping.
//!
//! Implements a Chamberlin SVF with LowPass, HighPass, BandPass, and Notch modes.
//! Used for: click shaping, body tone control, sub filtering, master EQ.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FilterMode {
    #[name = "Low Pass"]
    LowPass,
    #[name = "High Pass"]
    HighPass,
    #[name = "Band Pass"]
    BandPass,
    #[name = "Notch"]
    Notch,
}

/// Chamberlin state-variable filter.
///
/// Provides simultaneous LP, HP, BP, Notch outputs from a single topology.
/// Resonance is stable up to moderate values; the internal clamping prevents blowup.
pub struct SvfFilter {
    sample_rate: f32,
    cutoff: f32,
    resonance: f32,
    mode: FilterMode,
    // State variables
    ic1eq: f32,
    ic2eq: f32,
}

impl SvfFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            cutoff: 1000.0,
            resonance: 0.0,
            mode: FilterMode::LowPass,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    /// Set cutoff frequency in Hz. Clamped to safe range.
    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(20.0, self.sample_rate * 0.49);
    }

    /// Set resonance amount (0.0 = no resonance, 1.0 = maximum safe resonance).
    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = res.clamp(0.0, 1.0);
    }

    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    /// Process a single sample through the filter.
    ///
    /// Uses the Andy Cytomic (Vadim Zavalishin) SVF topology for stability.
    pub fn process(&mut self, input: f32) -> f32 {
        // Compute coefficients
        let g = (std::f32::consts::PI * self.cutoff / self.sample_rate).tan();
        // Q ranges from 0.5 (no resonance) to ~20 (high resonance)
        let q = 0.5 + self.resonance * 19.5;
        let k = 1.0 / q;

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        // Clamp state to prevent NaN/Inf from blowing up
        self.ic1eq = self.ic1eq.clamp(-10.0, 10.0);
        self.ic2eq = self.ic2eq.clamp(-10.0, 10.0);

        match self.mode {
            FilterMode::LowPass => v2,
            FilterMode::HighPass => input - k * v1 - v2,
            FilterMode::BandPass => v1,
            FilterMode::Notch => input - k * v1,
        }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }
}

/// Simple one-pole low-pass filter for smooth parameter changes and gentle filtering.
pub struct OnePoleFilter {
    state: f32,
    coeff: f32,
}

impl OnePoleFilter {
    pub fn new(sample_rate: f32, freq: f32) -> Self {
        Self {
            state: 0.0,
            coeff: Self::calc_coeff(sample_rate, freq),
        }
    }

    pub fn set_freq(&mut self, sample_rate: f32, freq: f32) {
        self.coeff = Self::calc_coeff(sample_rate, freq);
    }

    fn calc_coeff(sample_rate: f32, freq: f32) -> f32 {
        let fc = freq.min(sample_rate * 0.49);
        let x = (-std::f32::consts::TAU * fc / sample_rate).exp();
        1.0 - x
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        self.state += self.coeff * (input - self.state);
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }
}

/// Simple biquad filter for master EQ bands.
/// Implements a peaking EQ (bell) filter.
pub struct BiquadFilter {
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State (Direct Form II Transposed)
    z1: f32,
    z2: f32,
}

impl BiquadFilter {
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Configure as a peaking EQ filter.
    /// - `freq`: center frequency in Hz
    /// - `gain_db`: gain in dB (positive = boost, negative = cut)
    /// - `q`: quality factor (bandwidth). 0.7 is a good default for EQ.
    /// - `sample_rate`: current sample rate
    pub fn set_peaking_eq(&mut self, freq: f32, gain_db: f32, q: f32, sample_rate: f32) {
        if gain_db.abs() < 0.01 {
            // Unity — no processing needed
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = std::f32::consts::TAU * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        // Normalize by a0
        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    /// Process a single sample through the biquad.
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;

        // Prevent denormals
        if self.z1.abs() < 1e-15 {
            self.z1 = 0.0;
        }
        if self.z2.abs() < 1e-15 {
            self.z2 = 0.0;
        }

        output
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}
