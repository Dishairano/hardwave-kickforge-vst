//! Lightweight 2x oversampling for the distortion stage.
//!
//! Upsamples by 2x with linear interpolation, processes at the higher rate,
//! then downsamples by averaging adjacent samples. This reduces aliasing
//! from the nonlinear distortion without heavy computational cost.

pub struct Oversampler2x {
    /// Previous input sample for interpolation during upsampling.
    prev_input: f32,
}

impl Oversampler2x {
    pub fn new() -> Self {
        Self { prev_input: 0.0 }
    }

    /// Process one input sample through an oversampled distortion function.
    ///
    /// `process_fn` is called twice (once for each oversampled point) and should
    /// apply the distortion algorithm. The two results are averaged for the output.
    ///
    /// This is designed to wrap a distortion `process()` call:
    /// ```ignore
    /// let output = oversampler.process(input, |s| distortion.process(s));
    /// ```
    #[inline]
    pub fn process<F>(&mut self, input: f32, mut process_fn: F) -> f32
    where
        F: FnMut(f32) -> f32,
    {
        // Upsample: create two samples via linear interpolation
        let up0 = (self.prev_input + input) * 0.5; // midpoint between prev and current
        let up1 = input;                             // current sample

        self.prev_input = input;

        // Process both oversampled points
        let out0 = process_fn(up0);
        let out1 = process_fn(up1);

        // Downsample: average the two processed samples
        (out0 + out1) * 0.5
    }

    pub fn reset(&mut self) {
        self.prev_input = 0.0;
    }
}
