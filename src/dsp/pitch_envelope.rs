//! Enhanced pitch envelope for kick drum body.
//!
//! Sweeps from `start_freq` down to `end_freq` with configurable curve types.
//! Call `trigger()` on note-on, then `process()` per sample to get the
//! current frequency.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PitchCurve {
    /// Standard exponential sweep (classic)
    #[name = "Exponential"]
    Exponential,
    /// Logarithmic — slow start, fast end
    #[name = "Logarithmic"]
    Logarithmic,
    /// Linear frequency interpolation
    #[name = "Linear"]
    Linear,
    /// S-curve — smooth ease-in/ease-out
    #[name = "S-Curve"]
    SCurve,
    /// Punch — fast initial drop (80% in first 20% of time), slow settle.
    /// This is the KEY curve for hardstyle character.
    #[name = "Punch"]
    Punch,
}

pub struct PitchEnvelope {
    sample_rate: f32,
    start_freq: f32,
    end_freq: f32,
    /// Decay time in seconds.
    decay_time: f32,
    /// Current position in the envelope (0.0 = just triggered, grows toward 1.0).
    phase: f32,
    /// Per-sample phase increment.
    phase_inc: f32,
    /// Curve type for the pitch sweep.
    curve: PitchCurve,
    active: bool,
}

impl PitchEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        let decay_time = 0.15;
        Self {
            sample_rate,
            start_freq: 800.0,
            end_freq: 50.0,
            decay_time,
            phase: 0.0,
            phase_inc: 1.0 / (sample_rate * decay_time),
            curve: PitchCurve::Exponential,
            active: false,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.recalc_inc();
    }

    pub fn set_start_freq(&mut self, freq: f32) {
        self.start_freq = freq;
    }

    pub fn set_end_freq(&mut self, freq: f32) {
        self.end_freq = freq;
    }

    /// Set decay time in milliseconds.
    pub fn set_decay_ms(&mut self, ms: f32) {
        self.decay_time = ms / 1000.0;
        self.recalc_inc();
    }

    pub fn set_curve(&mut self, curve: PitchCurve) {
        self.curve = curve;
    }

    fn recalc_inc(&mut self) {
        let samples = self.sample_rate * self.decay_time;
        self.phase_inc = if samples > 0.0 { 1.0 / samples } else { 1.0 };
    }

    /// Trigger the envelope (call on note-on).
    pub fn trigger(&mut self) {
        self.phase = 0.0;
        self.active = true;
    }

    /// Returns the current frequency for this sample, advancing the envelope.
    pub fn process(&mut self) -> f32 {
        if !self.active {
            return self.end_freq;
        }

        let t = self.phase.min(1.0);
        let shaped_t = self.shape_time(t);

        // Interpolate in log-frequency space for musical pitch sweep
        let log_start = self.start_freq.ln();
        let log_end = self.end_freq.ln();
        let freq = (log_start + (log_end - log_start) * shaped_t).exp();

        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.active = false;
        }

        freq
    }

    /// Shape the linear time value according to the selected curve.
    /// Input and output both in [0, 1] range.
    #[inline]
    fn shape_time(&self, t: f32) -> f32 {
        match self.curve {
            PitchCurve::Exponential => {
                // Exponential: fast start, slow tail
                // Using exp curve: 1 - e^(-kt) normalized
                let k = 5.0_f32;
                (1.0 - (-k * t).exp()) / (1.0 - (-k).exp())
            }
            PitchCurve::Logarithmic => {
                // Logarithmic: slow start, fast end
                let k = 5.0_f32;
                let raw = (1.0 + t * (k.exp() - 1.0)).ln() / k;
                raw
            }
            PitchCurve::Linear => {
                // Linear: constant rate
                t
            }
            PitchCurve::SCurve => {
                // Smoothstep S-curve
                let t2 = t * t;
                3.0 * t2 - 2.0 * t2 * t
            }
            PitchCurve::Punch => {
                // Punch: 80% of pitch drop in first 20% of time, then slow settle.
                // This creates the characteristic hardstyle "thump into screech" feel.
                if t < 0.2 {
                    // Fast power curve in the first 20%
                    let local_t = t / 0.2; // normalize to [0, 1]
                    // Use a steep power curve to reach 0.8 at local_t=1.0
                    let shaped = 1.0 - (1.0 - local_t).powi(3);
                    shaped * 0.8
                } else {
                    // Slow ease-out for remaining 80% -> 100%
                    let local_t = (t - 0.2) / 0.8; // normalize to [0, 1]
                    let shaped = 1.0 - (1.0 - local_t).powi(2);
                    0.8 + shaped * 0.2
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.active = false;
    }
}
