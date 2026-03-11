//! Noise generator layer for adding texture to kicks.
//! Supports White, Pink, and Filtered noise types.

use nih_plug::prelude::Enum;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug)]
pub enum NoiseType {
    #[name = "White"]
    White,
    #[name = "Pink"]
    Pink,
    #[name = "Filtered"]
    Filtered,
}

pub struct NoiseGen {
    noise_type: NoiseType,
    // Pink noise state (Voss-McCartney)
    pink_b0: f32,
    pink_b1: f32,
    pink_b2: f32,
    pink_b3: f32,
    pink_b4: f32,
    pink_b5: f32,
    pink_b6: f32,
    // Filtered noise: one-pole LP state
    filter_state: f32,
    filter_coeff: f32,
    filter_freq: f32,
    sample_rate: f32,
    // xorshift PRNG
    rng_state: u32,
}

impl NoiseGen {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            noise_type: NoiseType::White,
            pink_b0: 0.0, pink_b1: 0.0, pink_b2: 0.0,
            pink_b3: 0.0, pink_b4: 0.0, pink_b5: 0.0, pink_b6: 0.0,
            filter_state: 0.0,
            filter_coeff: Self::calc_coeff(sample_rate, 2000.0),
            filter_freq: 2000.0,
            sample_rate,
            rng_state: 0xDEADBEEF,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.filter_coeff = Self::calc_coeff(sr, self.filter_freq);
    }

    pub fn set_noise_type(&mut self, nt: NoiseType) {
        self.noise_type = nt;
    }

    pub fn set_filter_freq(&mut self, freq: f32) {
        self.filter_freq = freq.clamp(20.0, self.sample_rate * 0.49);
        self.filter_coeff = Self::calc_coeff(self.sample_rate, self.filter_freq);
    }

    fn calc_coeff(sample_rate: f32, freq: f32) -> f32 {
        let fc = freq.min(sample_rate * 0.49);
        1.0 - (-std::f32::consts::TAU * fc / sample_rate).exp()
    }

    #[inline]
    fn next_white(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    #[inline]
    pub fn process(&mut self) -> f32 {
        let white = self.next_white();
        match self.noise_type {
            NoiseType::White => white,
            NoiseType::Pink => {
                // Paul Kellet's refined method
                self.pink_b0 = 0.99886 * self.pink_b0 + white * 0.0555179;
                self.pink_b1 = 0.99332 * self.pink_b1 + white * 0.0750759;
                self.pink_b2 = 0.96900 * self.pink_b2 + white * 0.1538520;
                self.pink_b3 = 0.86650 * self.pink_b3 + white * 0.3104856;
                self.pink_b4 = 0.55000 * self.pink_b4 + white * 0.5329522;
                self.pink_b5 = -0.7616 * self.pink_b5 - white * 0.0168980;
                let pink = self.pink_b0 + self.pink_b1 + self.pink_b2 + self.pink_b3
                    + self.pink_b4 + self.pink_b5 + self.pink_b6 + white * 0.5362;
                self.pink_b6 = white * 0.115926;
                pink * 0.11 // normalize
            }
            NoiseType::Filtered => {
                self.filter_state += self.filter_coeff * (white - self.filter_state);
                self.filter_state
            }
        }
    }

    pub fn reset(&mut self) {
        self.pink_b0 = 0.0; self.pink_b1 = 0.0; self.pink_b2 = 0.0;
        self.pink_b3 = 0.0; self.pink_b4 = 0.0; self.pink_b5 = 0.0;
        self.pink_b6 = 0.0;
        self.filter_state = 0.0;
    }
}
