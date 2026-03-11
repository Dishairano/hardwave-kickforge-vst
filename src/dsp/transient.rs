//! Transient shaper for the FX chain.
//! Controls attack punch and sustain level independently.

pub struct TransientShaper {
    sample_rate: f32,
    /// Envelope follower for detecting transients.
    envelope: f32,
    /// Attack emphasis (-1.0 to 1.0, 0 = neutral).
    attack: f32,
    /// Sustain emphasis (-1.0 to 1.0, 0 = neutral).
    sustain: f32,
    /// Fast envelope (for transient detection).
    env_fast: f32,
    /// Slow envelope (for sustained level).
    env_slow: f32,
    fast_attack_coeff: f32,
    fast_release_coeff: f32,
    slow_attack_coeff: f32,
    slow_release_coeff: f32,
}

impl TransientShaper {
    pub fn new(sample_rate: f32) -> Self {
        let mut s = Self {
            sample_rate,
            envelope: 0.0,
            attack: 0.0,
            sustain: 0.0,
            env_fast: 0.0,
            env_slow: 0.0,
            fast_attack_coeff: 0.0,
            fast_release_coeff: 0.0,
            slow_attack_coeff: 0.0,
            slow_release_coeff: 0.0,
        };
        s.calc_coeffs();
        s
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.calc_coeffs();
    }

    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack.clamp(-1.0, 1.0);
    }

    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(-1.0, 1.0);
    }

    fn calc_coeffs(&mut self) {
        // Fast envelope: 0.1ms attack, 5ms release
        self.fast_attack_coeff = (-1.0 / (self.sample_rate * 0.0001)).exp();
        self.fast_release_coeff = (-1.0 / (self.sample_rate * 0.005)).exp();
        // Slow envelope: 20ms attack, 200ms release
        self.slow_attack_coeff = (-1.0 / (self.sample_rate * 0.02)).exp();
        self.slow_release_coeff = (-1.0 / (self.sample_rate * 0.2)).exp();
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let abs_in = input.abs();

        // Fast follower
        let fast_coeff = if abs_in > self.env_fast {
            self.fast_attack_coeff
        } else {
            self.fast_release_coeff
        };
        self.env_fast = fast_coeff * self.env_fast + (1.0 - fast_coeff) * abs_in;

        // Slow follower
        let slow_coeff = if abs_in > self.env_slow {
            self.slow_attack_coeff
        } else {
            self.slow_release_coeff
        };
        self.env_slow = slow_coeff * self.env_slow + (1.0 - slow_coeff) * abs_in;

        // Transient = difference between fast and slow
        let transient = (self.env_fast - self.env_slow).max(0.0);
        // Sustained = slow envelope
        let sustained = self.env_slow;

        // Compute gain modifier
        let attack_gain = if self.env_slow > 0.0001 {
            1.0 + self.attack * 4.0 * (transient / (self.env_slow + 0.0001))
        } else {
            1.0
        };
        let sustain_gain = if sustained > 0.0001 && transient < self.env_slow * 0.5 {
            1.0 + self.sustain * 0.5
        } else {
            1.0
        };

        input * attack_gain * sustain_gain
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.env_fast = 0.0;
        self.env_slow = 0.0;
    }
}
