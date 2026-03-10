//! Exponential pitch envelope for kick drum body.
//!
//! Sweeps from `start_freq` down to `end_freq` with an exponential curve.
//! Call `trigger()` on note-on, then `process()` per sample to get the
//! current frequency.

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
    /// Uses exponential interpolation: freq = start * (end/start)^t
    pub fn process(&mut self) -> f32 {
        if !self.active {
            return self.end_freq;
        }

        let t = self.phase.min(1.0);
        // Exponential sweep: start_freq * (end_freq / start_freq) ^ t
        let ratio = self.end_freq / self.start_freq;
        let freq = self.start_freq * ratio.powf(t);

        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.active = false;
        }

        freq
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.active = false;
    }
}
