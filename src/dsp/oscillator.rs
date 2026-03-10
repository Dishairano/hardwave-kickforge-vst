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
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 50.0,
            waveform: Waveform::Sine,
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

    /// Generate one sample at the current frequency using the selected waveform.
    pub fn process(&mut self) -> f32 {
        let p = self.phase;
        let sample = match self.waveform {
            Waveform::Sine => (p * std::f32::consts::TAU).sin(),
            Waveform::Triangle => {
                // Triangle: rises linearly 0..0.5, falls 0.5..1.0
                if p < 0.5 {
                    4.0 * p - 1.0
                } else {
                    3.0 - 4.0 * p
                }
            }
            Waveform::Saw => {
                // Saw: rises from -1 to +1 over the period
                2.0 * p - 1.0
            }
            Waveform::Pulse => {
                // Pulse wave with 50% duty cycle
                if p < 0.5 { 1.0 } else { -1.0 }
            }
            Waveform::SineFold => {
                // Sine with foldback distortion: amplify sine then fold
                let raw = (p * std::f32::consts::TAU).sin() * 2.5;
                sine_fold(raw)
            }
        };

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
