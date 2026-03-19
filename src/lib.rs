//! KickForge — hardstyle & hardcore kick synthesizer VST3/CLAP instrument.
//!
//! Channel rack instrument: no audio input, stereo output, MIDI triggered.
//! - Audio thread: reads params, accepts MIDI note-on, synthesises kick sound
//! - Editor thread: wry WebView loaded from kickforge.hardwavestudios.com
//! - Plugin -> WebView: param state pushed at ~30Hz via crossbeam channel
//! - WebView -> Plugin: param changes via IPC -> GuiContext -> nih-plug params

use crossbeam_channel::{bounded, Receiver, Sender};
use nih_plug::prelude::*;
use parking_lot::Mutex;
use std::sync::Arc;

mod auth;
mod dsp;
#[cfg(feature = "gui")]
mod editor;
mod params;
mod presets;
mod protocol;

use dsp::click::Click;
use dsp::compressor::{Compressor, SoftLimiter};
use dsp::distortion::{Distortion, DistortionType};
use dsp::filter::{BiquadFilter, SvfFilter};
use dsp::noise::NoiseGen;
use dsp::oscillator::Oscillator;
use dsp::oversampling::Oversampler2x;
use dsp::pitch_envelope::PitchEnvelope;
use dsp::transient::TransientShaper;
use params::{KickForgeParams, NUM_FX_SLOTS, FX_EMPTY, FX_EQ, FX_COMP, FX_DIST, FX_TRANS};
use protocol::KickForgePacket;

/// How often we send param state to the editor (every N samples).
const EDITOR_UPDATE_INTERVAL: u32 = 512;

pub struct HardwaveKickForge {
    params: Arc<KickForgeParams>,

    // DSP — Body layer
    body_osc: Oscillator,
    body_pitch_env: PitchEnvelope,
    body_distortion: Distortion,
    body_oversampler: Oversampler2x,
    body_tone_filter: SvfFilter,
    body_amp_level: f32,
    body_amp_decay: f32,
    /// Hold counter: while > 0, body_amp_level stays at 1.0 (the "punch window").
    body_hold_remaining: u32,

    // DSP — Split-band distortion filters
    /// LP filter to extract clean sub before distortion
    split_lp: SvfFilter,
    /// HP filter to extract mid/hi for distortion
    split_hp: SvfFilter,

    // DSP — Click layer
    click: Click,

    // DSP — Sub layer (legacy — kept for struct compat, no longer used for synthesis)
    sub_osc: Oscillator,
    sub_amp_level: f32,
    #[allow(dead_code)]
    sub_amp_decay: f32,

    // DSP — Noise layer
    noise_gen: NoiseGen,
    noise_amp_level: f32,
    noise_amp_decay: f32,

    // DSP — Master
    eq_low: BiquadFilter,
    eq_mid: BiquadFilter,
    eq_high: BiquadFilter,
    limiter: SoftLimiter,

    // DSP — Modular FX slot pools (pre-allocated, one per slot)
    fx_eq_bands: [[BiquadFilter; 2]; NUM_FX_SLOTS],
    fx_comps: [Compressor; NUM_FX_SLOTS],
    fx_dists: [Distortion; NUM_FX_SLOTS],
    fx_trans: [TransientShaper; NUM_FX_SLOTS],

    // Velocity of last note-on (0.0 - 1.0)
    velocity: f32,

    // MIDI note number of last note-on (for pitch tracking)
    note_freq_ratio: f32,

    sample_rate: f32,

    // Plugin -> Editor communication
    editor_packet_tx: Sender<KickForgePacket>,
    editor_packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
    update_counter: u32,

    // Waveform buffer for visualizer (captures last kick)
    waveform_buf: Vec<f32>,
    waveform_write_pos: usize,
    waveform_capturing: bool,

    // Compressor gain reduction metering
    comp_gr_db: f32,
}

/// Calculate per-sample exponential decay coefficient from decay time in ms.
#[inline]
fn decay_coeff(sample_rate: f32, decay_ms: f32) -> f32 {
    let samples = sample_rate * decay_ms / 1000.0;
    if samples > 0.0 {
        0.001_f32.powf(1.0 / samples)
    } else {
        0.0
    }
}

impl Default for HardwaveKickForge {
    fn default() -> Self {
        let sr = 44100.0;
        let (pkt_tx, pkt_rx) = bounded::<KickForgePacket>(4);

        Self {
            params: Arc::new(KickForgeParams::default()),
            body_osc: Oscillator::new(sr),
            body_pitch_env: PitchEnvelope::new(sr),
            body_distortion: Distortion::new(),
            body_oversampler: Oversampler2x::new(),
            body_tone_filter: SvfFilter::new(sr),
            body_amp_level: 0.0,
            body_amp_decay: decay_coeff(sr, 500.0),
            body_hold_remaining: 0,
            split_lp: SvfFilter::new(sr),
            split_hp: SvfFilter::new(sr),
            click: Click::new(sr),
            sub_osc: Oscillator::new(sr),
            sub_amp_level: 0.0,
            sub_amp_decay: decay_coeff(sr, 300.0),
            noise_gen: NoiseGen::new(sr),
            noise_amp_level: 0.0,
            noise_amp_decay: decay_coeff(sr, 100.0),
            eq_low: BiquadFilter::new(),
            eq_mid: BiquadFilter::new(),
            eq_high: BiquadFilter::new(),
            limiter: SoftLimiter::new(),
            fx_eq_bands: std::array::from_fn(|_| [BiquadFilter::new(), BiquadFilter::new()]),
            fx_comps: std::array::from_fn(|_| Compressor::new(sr)),
            fx_dists: std::array::from_fn(|_| Distortion::new()),
            fx_trans: std::array::from_fn(|_| TransientShaper::new(sr)),
            velocity: 1.0,
            note_freq_ratio: 1.0,
            sample_rate: sr,
            editor_packet_tx: pkt_tx,
            editor_packet_rx: Arc::new(Mutex::new(pkt_rx)),
            update_counter: 0,
            waveform_buf: vec![0.0; 512],
            waveform_write_pos: 0,
            waveform_capturing: false,
            comp_gr_db: 0.0,
        }
    }
}

impl Plugin for HardwaveKickForge {
    const NAME: &'static str = "KickForge";
    const VENDOR: &'static str = "Hardwave Studios";
    const URL: &'static str = "https://hardwavestudios.com";
    const EMAIL: &'static str = "hello@hardwavestudios.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // Instrument: no audio input, stereo output (channel rack plugin)
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    #[cfg(feature = "gui")]
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let token = auth::load_token();
        Some(Box::new(editor::KickForgeEditor::new(
            Arc::clone(&self.params),
            Arc::clone(&self.editor_packet_rx),
            token,
        )))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.body_osc.set_sample_rate(self.sample_rate);
        self.body_pitch_env.set_sample_rate(self.sample_rate);
        self.body_tone_filter.set_sample_rate(self.sample_rate);
        self.split_lp.set_sample_rate(self.sample_rate);
        self.split_hp.set_sample_rate(self.sample_rate);
        self.click.set_sample_rate(self.sample_rate);
        self.sub_osc.set_sample_rate(self.sample_rate);
        self.noise_gen.set_sample_rate(self.sample_rate);
        for comp in self.fx_comps.iter_mut() {
            comp.set_sample_rate(self.sample_rate);
        }
        for ts in self.fx_trans.iter_mut() {
            ts.set_sample_rate(self.sample_rate);
        }
        true
    }

    fn reset(&mut self) {
        self.body_osc.reset();
        self.body_pitch_env.reset();
        self.body_distortion.reset();
        self.body_oversampler.reset();
        self.body_tone_filter.reset();
        self.body_amp_level = 0.0;
        self.body_hold_remaining = 0;
        self.split_lp.reset();
        self.split_hp.reset();
        self.click.reset();
        self.sub_osc.reset();
        self.sub_amp_level = 0.0;
        self.noise_gen.reset();
        self.noise_amp_level = 0.0;
        self.eq_low.reset();
        self.eq_mid.reset();
        self.eq_high.reset();
        for bands in self.fx_eq_bands.iter_mut() {
            bands[0].reset();
            bands[1].reset();
        }
        for comp in self.fx_comps.iter_mut() {
            comp.reset();
        }
        for dist in self.fx_dists.iter_mut() {
            dist.reset();
        }
        for ts in self.fx_trans.iter_mut() {
            ts.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // ── Read all param values ────────────────────────────────────────────
        let click_on = self.params.click_enabled.value();
        let click_type_val = self.params.click_type.value();
        let click_vol = self.params.click_volume.value();
        let click_pitch = self.params.click_pitch.value();
        let click_decay = self.params.click_decay.value();
        let click_filter = self.params.click_filter_freq.value();

        let body_pitch_start = self.params.body_pitch_start.value();
        let body_pitch_end = self.params.body_pitch_end.value();
        let body_pitch_decay = self.params.body_pitch_decay.value();
        let body_pitch_curve = self.params.body_pitch_curve.value();
        let body_waveform = self.params.body_waveform.value();
        let body_drive = self.params.body_drive.value();
        let body_dist_type = self.params.body_distortion_type.value();
        let body_decay_ms = self.params.body_decay.value();
        let body_vol = self.params.body_volume.value();
        let body_tone = self.params.body_tone.value();
        let body_resonance = self.params.body_resonance.value();
        let body_feedback = self.params.body_feedback.value();
        let body_hold_ms = self.params.body_hold.value();
        let body_split_freq = self.params.body_split_freq.value();

        let sub_on = self.params.sub_enabled.value();
        let sub_vol = self.params.sub_volume.value();
        // Legacy params — read but not used for separate osc anymore
        let _sub_freq = self.params.sub_frequency.value();
        let _sub_decay_ms = self.params.sub_decay.value();

        let noise_on = self.params.noise_enabled.value();
        let noise_type = self.params.noise_type.value();
        let noise_vol = self.params.noise_volume.value();
        let noise_decay_ms = self.params.noise_decay.value();
        let noise_filter = self.params.noise_filter_freq.value();

        let click_solo = self.params.click_solo.value();
        let body_solo = self.params.body_solo.value();
        let sub_solo = self.params.sub_solo.value();
        let noise_solo = self.params.noise_solo.value();
        let any_solo = click_solo || body_solo || sub_solo || noise_solo;

        let vel_to_decay = self.params.vel_to_decay.value();
        let vel_to_pitch = self.params.vel_to_pitch.value();
        let vel_to_drive = self.params.vel_to_drive.value();
        let vel_to_click = self.params.vel_to_click.value();

        let master_vol = self.params.master_volume.value();
        let master_tuning = self.params.master_tuning.value();
        let master_octave = self.params.master_octave.value();
        let limiter_on = self.params.master_limiter.value();
        let eq_low_db = self.params.master_low.value();
        let eq_mid_db = self.params.master_mid.value();
        let eq_high_db = self.params.master_high.value();

        // ── Read FX slot params ──────────────────────────────────────────────
        let mut slot_types = [0i32; NUM_FX_SLOTS];
        let mut slot_enabled = [false; NUM_FX_SLOTS];
        let mut slot_p = [[0.0f32; 6]; NUM_FX_SLOTS];
        for i in 0..NUM_FX_SLOTS {
            slot_types[i] = self.params.fx_slots[i].slot_type.value();
            slot_enabled[i] = self.params.fx_slots[i].enabled.value();
            slot_p[i][0] = self.params.fx_slots[i].p1.value();
            slot_p[i][1] = self.params.fx_slots[i].p2.value();
            slot_p[i][2] = self.params.fx_slots[i].p3.value();
            slot_p[i][3] = self.params.fx_slots[i].p4.value();
            slot_p[i][4] = self.params.fx_slots[i].p5.value();
            slot_p[i][5] = self.params.fx_slots[i].p6.value();
        }

        // ── Update DSP state from params ─────────────────────────────────────
        // Velocity-modulated values
        let vel = self.velocity;
        let eff_drive = (body_drive + vel_to_drive * (vel - 0.5)).clamp(0.0, 1.0);
        let eff_pitch_start = body_pitch_start * (1.0 + vel_to_pitch * (vel - 0.5));
        let eff_decay_ms = body_decay_ms * (1.0 + vel_to_decay * (vel - 0.5));
        let eff_click_vol = (click_vol + vel_to_click * (vel - 0.5)).clamp(0.0, 1.0);

        // Combine octave, fine tuning (semitones), and MIDI note tracking
        let tuning_factor = 2.0_f32.powf((master_tuning + master_octave as f32 * 12.0) / 12.0)
            * self.note_freq_ratio;
        self.body_pitch_env
            .set_start_freq(eff_pitch_start * tuning_factor);
        self.body_pitch_env
            .set_end_freq(body_pitch_end * tuning_factor);
        self.body_pitch_env.set_decay_ms(body_pitch_decay);
        self.body_pitch_env.set_curve(body_pitch_curve);

        self.body_osc.set_waveform(body_waveform);
        self.body_osc.set_feedback(body_feedback);
        self.body_distortion.set_drive(eff_drive);
        self.body_distortion.set_type(body_dist_type);
        self.body_amp_decay = decay_coeff(self.sample_rate, eff_decay_ms);

        self.body_tone_filter.set_cutoff(body_tone);
        self.body_tone_filter.set_resonance(body_resonance);

        // Split-band distortion: LP extracts clean sub, HP extracts mid/hi
        {
            use crate::dsp::filter::FilterMode;
            self.split_lp.set_cutoff(body_split_freq);
            self.split_lp.set_mode(FilterMode::LowPass);
            self.split_lp.set_resonance(0.0);
            self.split_hp.set_cutoff(body_split_freq);
            self.split_hp.set_mode(FilterMode::HighPass);
            self.split_hp.set_resonance(0.0);
        }

        self.click.set_click_type(click_type_val);
        self.click.set_pitch(click_pitch);
        self.click.set_decay_ms(click_decay);
        self.click.set_filter_freq(click_filter);

        self.noise_gen.set_noise_type(noise_type);
        self.noise_gen.set_filter_freq(noise_filter);
        self.noise_amp_decay = decay_coeff(self.sample_rate, noise_decay_ms);

        // Master EQ (peaking bands at 80, 1000, 8000 Hz)
        self.eq_low
            .set_peaking_eq(80.0, eq_low_db, 0.7, self.sample_rate);
        self.eq_mid
            .set_peaking_eq(1000.0, eq_mid_db, 0.7, self.sample_rate);
        self.eq_high
            .set_peaking_eq(8000.0, eq_high_db, 0.7, self.sample_rate);

        // ── Configure active FX slots ────────────────────────────────────────
        for i in 0..NUM_FX_SLOTS {
            if !slot_enabled[i] || slot_types[i] == FX_EMPTY {
                continue;
            }
            let p = &slot_p[i];
            match slot_types[i] {
                FX_EQ => {
                    // p1=freq1 (20-20kHz log), p2=gain1 (-12..+12dB), p3=q1 (0.1-10)
                    // p4=freq2, p5=gain2, p6=q2
                    let freq1 = 20.0 * (1000.0_f32).powf(p[0]);
                    let gain1 = p[1] * 24.0 - 12.0;
                    let q1 = 0.1 + p[2] * 9.9;
                    let freq2 = 20.0 * (1000.0_f32).powf(p[3]);
                    let gain2 = p[4] * 24.0 - 12.0;
                    let q2 = 0.1 + p[5] * 9.9;
                    self.fx_eq_bands[i][0].set_peaking_eq(freq1, gain1, q1, self.sample_rate);
                    self.fx_eq_bands[i][1].set_peaking_eq(freq2, gain2, q2, self.sample_rate);
                }
                FX_COMP => {
                    // p1=threshold (-60..0 dB), p2=ratio (1-20), p3=attack (0.1-100ms)
                    // p4=release (10-1000ms), p5=makeup (0-24dB)
                    let thresh_db = p[0] * -60.0;
                    let thresh_linear = 10.0_f32.powf(thresh_db / 20.0);
                    let ratio = 1.0 + p[1] * 19.0;
                    let attack_ms = 0.1 + p[2] * 99.9;
                    let release_ms = 10.0 + p[3] * 990.0;
                    self.fx_comps[i].set_threshold(thresh_linear);
                    self.fx_comps[i].set_ratio(ratio);
                    self.fx_comps[i].set_attack_ms(attack_ms);
                    self.fx_comps[i].set_release_ms(release_ms);
                }
                FX_DIST => {
                    // p1=type (0-4 mapped), p2=drive (0-1), p3=mix (0-1)
                    let dist_type = match (p[0] * 4.0).round() as usize {
                        0 => DistortionType::Tanh,
                        1 => DistortionType::HardClip,
                        2 => DistortionType::Foldback,
                        3 => DistortionType::Asymmetric,
                        _ => DistortionType::Bitcrush,
                    };
                    self.fx_dists[i].set_type(dist_type);
                    self.fx_dists[i].set_drive(p[1]);
                }
                FX_TRANS => {
                    // p1=attack (-1..1), p2=sustain (-1..1)
                    let attack = p[0] * 2.0 - 1.0;
                    let sustain = p[1] * 2.0 - 1.0;
                    self.fx_trans[i].set_attack(attack);
                    self.fx_trans[i].set_sustain(sustain);
                }
                _ => {}
            }
        }

        // ── Process audio ────────────────────────────────────────────────────
        let num_samples = buffer.samples();
        let mut next_event = context.next_event();
        let mut sample_idx = 0;

        let output = buffer.as_slice();
        let (left, right_and_rest) = output.split_first_mut().unwrap();
        let right = &mut right_and_rest[0];

        while sample_idx < num_samples {
            // Handle MIDI events at or before this sample
            while let Some(event) = next_event {
                if event.timing() > sample_idx as u32 {
                    break;
                }

                if let NoteEvent::NoteOn { velocity: vel, note, .. } = event {
                    self.velocity = vel;
                    // Track MIDI note pitch: C5 (note 60) = neutral (ratio 1.0)
                    self.note_freq_ratio = 2.0_f32.powf((note as f32 - 60.0) / 12.0);

                    // Trigger body with hold
                    self.body_osc.reset();
                    self.body_pitch_env.trigger();
                    self.body_amp_level = 1.0;
                    self.body_hold_remaining =
                        (self.sample_rate * body_hold_ms / 1000.0) as u32;

                    // Reset split filters on retrigger for clean transient
                    self.split_lp.reset();
                    self.split_hp.reset();

                    // Trigger click
                    if click_on {
                        self.click.trigger();
                    }

                    // Trigger noise
                    if noise_on {
                        self.noise_gen.reset();
                        self.noise_amp_level = 1.0;
                    }

                    // Start waveform capture
                    self.waveform_capturing = true;
                    self.waveform_write_pos = 0;
                }

                next_event = context.next_event();
            }

            // ── Body: pitch env → osc (with FM feedback) → split-band distortion ──
            let body_freq = self.body_pitch_env.process();
            self.body_osc.set_frequency(body_freq);
            let body_raw = self.body_osc.process();

            // Split-band distortion: LP extracts low frequencies to bypass
            // distortion (stays clean), HP extracts mid/hi which gets distorted.
            // The full body signal is always present — split-band only controls
            // *what gets distorted*, not what is audible.
            let clean_sub = self.split_lp.process(body_raw);
            let mid_hi = self.split_hp.process(body_raw);

            // Oversample the mid/hi through distortion to reduce aliasing
            let mid_hi_distorted = self
                .body_oversampler
                .process(mid_hi, |s| self.body_distortion.process(s));

            // Tone filter (low-pass) shapes the distorted upper content
            let mid_hi_filtered = self.body_tone_filter.process(mid_hi_distorted);

            // Recombine: clean sub + distorted mids/highs = full body
            // Sub volume controls how much clean low-end is blended in.
            // When sub is off, the clean_sub portion is still included at
            // unity to preserve the full body signal — sub_vol only boosts it.
            let sub_gain = if sub_on { sub_vol } else { 1.0 };
            let body_combined = mid_hi_filtered + clean_sub * sub_gain;

            // Amp envelope with hold: during hold, level stays at 1.0
            let body_sample = body_combined * self.body_amp_level * body_vol;
            if self.body_hold_remaining > 0 {
                self.body_hold_remaining -= 1;
                // body_amp_level stays at 1.0 during hold
            } else {
                self.body_amp_level *= self.body_amp_decay;
                if self.body_amp_level < 0.0001 {
                    self.body_amp_level = 0.0;
                }
            }

            // ── Click transient ──
            let click_sample = if click_on {
                self.click.process() * eff_click_vol
            } else {
                0.0
            };

            // ── Noise ──
            let noise_sample = if noise_on {
                let s = self.noise_gen.process() * self.noise_amp_level * noise_vol;
                self.noise_amp_level *= self.noise_amp_decay;
                if self.noise_amp_level < 0.0001 {
                    self.noise_amp_level = 0.0;
                }
                s
            } else {
                0.0
            };

            // ── Mix layers with solo logic ──
            // Sub is now part of the body (via split-band), not a separate layer
            let mut mixed = if any_solo {
                let mut m = 0.0;
                if click_solo { m += click_sample; }
                if body_solo || sub_solo { m += body_sample; }
                if noise_solo { m += noise_sample; }
                m * master_vol * vel
            } else {
                (body_sample + click_sample + noise_sample) * master_vol * vel
            };

            // ── Master 3-band EQ ──
            mixed = self.eq_low.process(mixed);
            mixed = self.eq_mid.process(mixed);
            mixed = self.eq_high.process(mixed);

            // ── Modular FX rack (process slots in order) ──
            for i in 0..NUM_FX_SLOTS {
                if !slot_enabled[i] || slot_types[i] == FX_EMPTY {
                    continue;
                }
                match slot_types[i] {
                    FX_EQ => {
                        mixed = self.fx_eq_bands[i][0].process(mixed);
                        mixed = self.fx_eq_bands[i][1].process(mixed);
                    }
                    FX_COMP => {
                        let pre = mixed.abs();
                        mixed = self.fx_comps[i].process(mixed);
                        let post = mixed.abs();
                        if pre > 0.0001 {
                            let gr = 20.0 * (post / pre).log10();
                            self.comp_gr_db = self.comp_gr_db * 0.95 + gr * 0.05;
                        }
                        // Makeup gain from p5
                        let makeup_db = slot_p[i][4] * 24.0;
                        let makeup_linear = 10.0_f32.powf(makeup_db / 20.0);
                        mixed *= makeup_linear;
                    }
                    FX_DIST => {
                        let dry = mixed;
                        let wet = self.fx_dists[i].process(mixed);
                        let mix_amt = slot_p[i][2]; // p3 = mix
                        mixed = dry * (1.0 - mix_amt) + wet * mix_amt;
                    }
                    FX_TRANS => {
                        mixed = self.fx_trans[i].process(mixed);
                    }
                    _ => {}
                }
            }

            // ── Soft limiter ──
            if limiter_on {
                mixed = self.limiter.process(mixed);
            }

            // Write mono kick to stereo output
            left[sample_idx] = mixed;
            right[sample_idx] = mixed;

            // Capture waveform for visualizer
            if self.waveform_capturing && self.waveform_write_pos < self.waveform_buf.len() {
                self.waveform_buf[self.waveform_write_pos] = mixed;
                self.waveform_write_pos += 1;
                if self.waveform_write_pos >= self.waveform_buf.len() {
                    self.waveform_capturing = false;
                }
            }

            sample_idx += 1;
        }

        // ── Throttled editor update ──────────────────────────────────────────
        self.update_counter += num_samples as u32;
        if self.update_counter >= EDITOR_UPDATE_INTERVAL {
            self.update_counter = 0;
            let packet = KickForgePacket {
                click_enabled: click_on,
                click_type: click_type_val as i32,
                click_volume: click_vol,
                click_pitch,
                click_decay,
                click_filter_freq: click_filter,
                body_pitch_start,
                body_pitch_end,
                body_pitch_decay,
                body_pitch_curve: body_pitch_curve as i32,
                body_waveform: body_waveform as i32,
                body_drive,
                body_distortion_type: body_dist_type as i32,
                body_decay: body_decay_ms,
                body_volume: body_vol,
                body_tone,
                body_resonance,
                body_feedback,
                body_hold: body_hold_ms,
                body_split_freq,
                sub_enabled: sub_on,
                sub_frequency: _sub_freq,
                sub_volume: sub_vol,
                sub_decay: _sub_decay_ms,
                noise_enabled: noise_on,
                noise_type: noise_type as i32,
                noise_volume: noise_vol,
                noise_decay: noise_decay_ms,
                noise_filter_freq: noise_filter,
                click_solo, body_solo, sub_solo, noise_solo,
                vel_to_decay, vel_to_pitch, vel_to_drive, vel_to_click,
                fx_slots: (0..NUM_FX_SLOTS).map(|i| protocol::FxSlotState {
                    slot_type: slot_types[i],
                    enabled: slot_enabled[i],
                    p1: slot_p[i][0],
                    p2: slot_p[i][1],
                    p3: slot_p[i][2],
                    p4: slot_p[i][3],
                    p5: slot_p[i][4],
                    p6: slot_p[i][5],
                }).collect(),
                comp_gain_reduction: self.comp_gr_db,
                waveform_buffer: self.waveform_buf.clone(),
                master_volume: master_vol,
                master_tuning,
                master_octave: master_octave as i32,
                master_limiter: limiter_on,
                master_low: eq_low_db,
                master_mid: eq_mid_db,
                master_high: eq_high_db,
            };
            let _ = self.editor_packet_tx.try_send(packet);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for HardwaveKickForge {
    const CLAP_ID: &'static str = "com.hardwavestudios.kickforge";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Hardstyle & hardcore kick synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for HardwaveKickForge {
    const VST3_CLASS_ID: [u8; 16] = *b"HWKickForge_v001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
    ];
}

nih_export_clap!(HardwaveKickForge);
nih_export_vst3!(HardwaveKickForge);
