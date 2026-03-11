//! Multi-mode distortion for kick body drive.
//!
//! Supports Tanh (soft saturation), HardClip, Foldback (wave folding),
//! Asymmetric (soft clip with even harmonics), and Bitcrush.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum DistortionType {
    /// Soft saturation via tanh — warm, musical
    #[name = "Tanh"]
    Tanh,
    /// Hard clipper — harsh, digital
    #[name = "Hard Clip"]
    HardClip,
    /// Wave folding — complex harmonics, great for screech
    #[name = "Foldback"]
    Foldback,
    /// Asymmetric soft clip — adds even harmonics
    #[name = "Asymmetric"]
    Asymmetric,
    /// Bit reduction + sample rate reduction — lo-fi character
    #[name = "Bitcrush"]
    Bitcrush,
}

pub struct Distortion {
    /// Drive amount 0.0..1.0, mapped to a gain multiplier internally.
    drive: f32,
    /// Post-distortion output gain 0.0..1.0.
    post_gain: f32,
    /// Active distortion algorithm.
    distortion_type: DistortionType,
    /// Bitcrush: held sample value for sample rate reduction.
    bc_hold: f32,
    /// Bitcrush: counter for sample-and-hold.
    bc_counter: f32,
}

impl Distortion {
    pub fn new() -> Self {
        Self {
            drive: 0.0,
            post_gain: 1.0,
            distortion_type: DistortionType::Tanh,
            bc_hold: 0.0,
            bc_counter: 0.0,
        }
    }

    /// Set drive amount (0.0 = clean, 1.0 = heavy saturation).
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Set post-distortion output gain (0.0 .. 1.0).
    pub fn set_post_gain(&mut self, gain: f32) {
        self.post_gain = gain.clamp(0.0, 2.0);
    }

    /// Set the distortion algorithm.
    pub fn set_type(&mut self, dtype: DistortionType) {
        self.distortion_type = dtype;
    }

    /// Process a single sample through the selected waveshaper.
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.drive < 0.001 {
            return sample * self.post_gain;
        }

        // Map drive 0..1 to gain 1..30 (more headroom for aggressive algorithms)
        let gain = 1.0 + self.drive * 29.0;
        let driven = sample * gain;

        let shaped = match self.distortion_type {
            DistortionType::Tanh => {
                // Soft saturation
                driven.tanh()
            }
            DistortionType::HardClip => {
                // Hard clip at [-1, 1]
                driven.clamp(-1.0, 1.0)
            }
            DistortionType::Foldback => {
                // Wave folding: fold back at [-1, 1] boundaries
                wave_fold(driven)
            }
            DistortionType::Asymmetric => {
                // Asymmetric soft clip: different curves for positive/negative
                // Adds even harmonics which thicken the sound
                if driven >= 0.0 {
                    // Softer positive clip
                    (driven * 1.5).tanh() * 0.8
                } else {
                    // Harder negative clip — creates asymmetry
                    let x = driven * 2.0;
                    -(x * x).min(1.0) * driven.signum() * 0.9
                }
            }
            DistortionType::Bitcrush => {
                // Bit reduction: quantize to fewer levels
                // Drive controls both bit depth and sample rate reduction
                let bits = 16.0 - self.drive * 12.0; // 16 bits down to 4 bits
                let levels = (2.0_f32).powf(bits);
                let quantized = (driven.clamp(-1.0, 1.0) * levels).round() / levels;

                // Sample rate reduction: hold samples
                let sr_factor = 1.0 + self.drive * 15.0; // 1x to 16x reduction
                self.bc_counter += 1.0;
                if self.bc_counter >= sr_factor {
                    self.bc_counter = 0.0;
                    self.bc_hold = quantized;
                }
                self.bc_hold
            }
        };

        // Compensate for volume increase from saturation
        let compensation = 1.0 / (1.0 + self.drive * 0.5);
        shaped * compensation * self.post_gain
    }

    pub fn reset(&mut self) {
        self.bc_hold = 0.0;
        self.bc_counter = 0.0;
    }
}

/// Wave fold: reflects the signal at [-1, 1] boundaries, creating complex
/// harmonics. This is the key algorithm for hardstyle "screech" sounds.
#[inline]
fn wave_fold(x: f32) -> f32 {
    // Normalize input into [-1, 1] by folding at boundaries
    // Period is 4 (goes -1 -> 1 -> -1 in 4 units)
    let x = x + 1.0; // shift to [0, ...]
    let period = 4.0;
    let mut pos = x % period;
    if pos < 0.0 {
        pos += period;
    }
    // Triangle wave shape: 0->2 maps to -1->1, 2->4 maps to 1->-1
    if pos < 2.0 {
        pos - 1.0
    } else {
        3.0 - pos
    }
}
