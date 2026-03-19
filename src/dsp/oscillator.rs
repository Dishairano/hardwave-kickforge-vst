//! Multi-mode oscillator for kick synthesis.
//!
//! Supports Sine, Triangle, Saw, Pulse, and SineFold (sine with foldback)
//! waveforms. Maintains phase continuity across frequency changes for smooth
//! pitch sweeps.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Waveform {
    #[name = "Sine"]
    Sine,
    #[name = "Triangle"]
    Triangle,
    #[name = "Saw"]
    Saw,
    #[name = "Pulse"]
    Pulse,
    #[name = "SineFold"]
    SineFold,
}

pub struct Oscillator {
    sample_rate: f32,
    phase: f32,
    frequency: f32,
    waveform: Waveform,
    /// FM feedback: previous output fed back into phase for self-modulation.
    feedback_amount: f32,
    feedback_sample: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 50.0,
            waveform: Waveform::Sine,
            feedback_amount: 0.0,
            feedback_sample: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn set_waveform(&mut self, wf: Waveform) {
        self.waveform = wf;
    }

    /// Set FM self-modulation amount (0.0 = clean, 1.0 = heavy).
    /// Feeds the oscillator output back into its own phase, thickening the
    /// tone with additional harmonics before it even reaches distortion.
    pub fn set_feedback(&mut self, amount: f32) {
        self.feedback_amount = amount.clamp(0.0, 1.0);
    }

    /// Generate one sample at the current frequency using the selected waveform.
    pub fn process(&mut self) -> f32 {
        // Apply self-modulation: shift phase by previous output * feedback amount.
        // Scale feedback to max ~1.5 radians / TAU for musical range.
        let fb_phase = self.feedback_sample * self.feedback_amount * 0.24;
        let p = (self.phase + fb_phase).fract().abs();

        let sample = match self.waveform {
            Waveform::Sine => (p * std::f32::consts::TAU).sin(),
            Waveform::Triangle => {
                if p < 0.5 {
                    4.0 * p - 1.0
                } else {
                    3.0 - 4.0 * p
                }
            }
            Waveform::Saw => {
                2.0 * p - 1.0
            }
            Waveform::Pulse => {
                if p < 0.5 { 1.0 } else { -1.0 }
            }
            Waveform::SineFold => {
                let raw = (p * std::f32::consts::TAU).sin() * 2.5;
                sine_fold(raw)
            }
        };

        self.feedback_sample = sample;
        self.phase += self.frequency / self.sample_rate;
        self.phase -= self.phase.floor();

        sample
    }

    /// Reset the oscillator phase (call on note-on for consistent attack).
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.feedback_sample = 0.0;
    }
}

/// Fold a signal back into the [-1, 1] range by reflecting at boundaries.
/// This creates complex harmonics when the input exceeds [-1, 1].
#[inline]
fn sine_fold(x: f32) -> f32 {
    // Normalize to [0, 4] period, then fold
    let mut v = x;
    // Wrap into [-1, 1] by folding
    while v > 1.0 {
        v = 2.0 - v;
    }
    while v < -1.0 {
        v = -2.0 - v;
    }
    v
}
