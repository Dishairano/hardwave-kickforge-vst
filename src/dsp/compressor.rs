//! Simple compressor / soft limiter for the master output.
//!
//! Provides threshold, ratio, attack, and release controls.
//! Used on the master bus to tame transients and add cohesion.

pub struct Compressor {
    sample_rate: f32,
    /// Threshold in linear amplitude (not dB).
    threshold: f32,
    /// Compression ratio (e.g. 4.0 = 4:1).
    ratio: f32,
    /// Attack coefficient (per-sample).
    attack_coeff: f32,
    /// Release coefficient (per-sample).
    release_coeff: f32,
    /// Current envelope follower level.
    envelope: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            threshold: 0.8,
            ratio: 4.0,
            attack_coeff: Self::calc_coeff(sample_rate, 1.0),   // 1ms attack
            release_coeff: Self::calc_coeff(sample_rate, 50.0), // 50ms release
            envelope: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    /// Set threshold in linear amplitude (0.0 - 1.0).
    pub fn set_threshold(&mut self, thresh: f32) {
        self.threshold = thresh.clamp(0.01, 1.0);
    }

    /// Set compression ratio (1.0 = no compression, inf = limiter).
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }

    /// Set attack time in milliseconds.
    pub fn set_attack_ms(&mut self, ms: f32) {
        self.attack_coeff = Self::calc_coeff(self.sample_rate, ms);
    }

    /// Set release time in milliseconds.
    pub fn set_release_ms(&mut self, ms: f32) {
        self.release_coeff = Self::calc_coeff(self.sample_rate, ms);
    }

    fn calc_coeff(sample_rate: f32, ms: f32) -> f32 {
        let samples = sample_rate * ms / 1000.0;
        if samples > 0.0 {
            (-1.0 / samples).exp()
        } else {
            0.0
        }
    }

    /// Process a single sample. Returns the compressed output.
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let abs_input = input.abs();

        // Envelope follower with attack/release
        let coeff = if abs_input > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * abs_input;

        // Compute gain reduction
        if self.envelope > self.threshold {
            let over_db = 20.0 * (self.envelope / self.threshold).log10();
            let compressed_db = over_db * (1.0 - 1.0 / self.ratio);
            let gain_reduction = 10.0_f32.powf(-compressed_db / 20.0);
            input * gain_reduction
        } else {
            input
        }
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

/// Ultra-simple soft limiter (tanh-based) for the final output stage.
/// Used when the master_limiter param is enabled.
pub struct SoftLimiter {
    /// Ceiling level in linear amplitude.
    ceiling: f32,
}

impl SoftLimiter {
    pub fn new() -> Self {
        Self { ceiling: 0.95 }
    }

    /// Process a single sample through the soft limiter.
    #[inline]
    pub fn process(&self, input: f32) -> f32 {
        // Scale to push signal toward tanh saturation at ceiling
        let scaled = input / self.ceiling;
        scaled.tanh() * self.ceiling
    }

    pub fn reset(&self) {
        // Stateless
    }
}
