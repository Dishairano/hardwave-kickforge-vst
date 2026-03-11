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
use dsp::distortion::Distortion;
use dsp::filter::{BiquadFilter, SvfFilter};
use dsp::oscillator::Oscillator;
use dsp::oversampling::Oversampler2x;
use dsp::pitch_envelope::PitchEnvelope;
use params::KickForgeParams;
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

    // DSP — Click layer
    click: Click,

    // DSP — Sub layer
    sub_osc: Oscillator,
    sub_amp_level: f32,
    sub_amp_decay: f32,

    // DSP — Master
    eq_low: BiquadFilter,
    eq_mid: BiquadFilter,
    eq_high: BiquadFilter,
    limiter: SoftLimiter,

    // DSP — FX Chain
    fx_eq1: BiquadFilter,
    fx_eq2: BiquadFilter,
    fx_eq3: BiquadFilter,
    fx_eq4: BiquadFilter,
    fx_compressor: Compressor,
    fx_distortion: Distortion,

    // Velocity of last note-on (0.0 - 1.0)
    velocity: f32,

    // MIDI note number of last note-on (for pitch tracking)
    note_freq_ratio: f32,

    sample_rate: f32,

    // Plugin -> Editor communication
    editor_packet_tx: Sender<KickForgePacket>,
    editor_packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,
    update_counter: u32,
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
            click: Click::new(sr),
            sub_osc: Oscillator::new(sr),
            sub_amp_level: 0.0,
            sub_amp_decay: decay_coeff(sr, 300.0),
            eq_low: BiquadFilter::new(),
            eq_mid: BiquadFilter::new(),
            eq_high: BiquadFilter::new(),
            limiter: SoftLimiter::new(),
            fx_eq1: BiquadFilter::new(),
            fx_eq2: BiquadFilter::new(),
            fx_eq3: BiquadFilter::new(),
            fx_eq4: BiquadFilter::new(),
            fx_compressor: Compressor::new(sr),
            fx_distortion: Distortion::new(),
            velocity: 1.0,
            note_freq_ratio: 1.0,
            sample_rate: sr,
            editor_packet_tx: pkt_tx,
            editor_packet_rx: Arc::new(Mutex::new(pkt_rx)),
            update_counter: 0,
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
        self.click.set_sample_rate(self.sample_rate);
        self.sub_osc.set_sample_rate(self.sample_rate);
        self.fx_compressor.set_sample_rate(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        self.body_osc.reset();
        self.body_pitch_env.reset();
        self.body_distortion.reset();
        self.body_oversampler.reset();
        self.body_tone_filter.reset();
        self.body_amp_level = 0.0;
        self.click.reset();
        self.sub_osc.reset();
        self.sub_amp_level = 0.0;
        self.eq_low.reset();
        self.eq_mid.reset();
        self.eq_high.reset();
        self.fx_eq1.reset();
        self.fx_eq2.reset();
        self.fx_eq3.reset();
        self.fx_eq4.reset();
        self.fx_compressor.reset();
        self.fx_distortion.reset();
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

        let sub_on = self.params.sub_enabled.value();
        let sub_freq = self.params.sub_frequency.value();
        let sub_vol = self.params.sub_volume.value();
        let sub_decay_ms = self.params.sub_decay.value();

        let master_vol = self.params.master_volume.value();
        let master_tuning = self.params.master_tuning.value();
        let master_octave = self.params.master_octave.value();
        let limiter_on = self.params.master_limiter.value();
        let eq_low_db = self.params.master_low.value();
        let eq_mid_db = self.params.master_mid.value();
        let eq_high_db = self.params.master_high.value();

        // FX params
        let fx_eq_on = self.params.fx_eq_enabled.value();
        let fx_eq1_freq = self.params.fx_eq1_freq.value();
        let fx_eq1_gain = self.params.fx_eq1_gain.value();
        let fx_eq1_q = self.params.fx_eq1_q.value();
        let fx_eq2_freq = self.params.fx_eq2_freq.value();
        let fx_eq2_gain = self.params.fx_eq2_gain.value();
        let fx_eq2_q = self.params.fx_eq2_q.value();
        let fx_eq3_freq = self.params.fx_eq3_freq.value();
        let fx_eq3_gain = self.params.fx_eq3_gain.value();
        let fx_eq3_q = self.params.fx_eq3_q.value();
        let fx_eq4_freq = self.params.fx_eq4_freq.value();
        let fx_eq4_gain = self.params.fx_eq4_gain.value();
        let fx_eq4_q = self.params.fx_eq4_q.value();

        let fx_comp_on = self.params.fx_comp_enabled.value();
        let fx_comp_thresh_db = self.params.fx_comp_threshold.value();
        let fx_comp_ratio = self.params.fx_comp_ratio.value();
        let fx_comp_attack = self.params.fx_comp_attack.value();
        let fx_comp_release = self.params.fx_comp_release.value();
        let fx_comp_makeup = self.params.fx_comp_makeup.value();

        let fx_dist_on = self.params.fx_dist_enabled.value();
        let fx_dist_type = self.params.fx_dist_type.value();
        let fx_dist_drive = self.params.fx_dist_drive.value();
        let fx_dist_mix = self.params.fx_dist_mix.value();

        // ── Update DSP state from params ─────────────────────────────────────
        // Combine octave, fine tuning (semitones), and MIDI note tracking
        let tuning_factor = 2.0_f32.powf((master_tuning + master_octave as f32 * 12.0) / 12.0)
            * self.note_freq_ratio;
        self.body_pitch_env
            .set_start_freq(body_pitch_start * tuning_factor);
        self.body_pitch_env
            .set_end_freq(body_pitch_end * tuning_factor);
        self.body_pitch_env.set_decay_ms(body_pitch_decay);
        self.body_pitch_env.set_curve(body_pitch_curve);

        self.body_osc.set_waveform(body_waveform);
        self.body_distortion.set_drive(body_drive);
        self.body_distortion.set_type(body_dist_type);
        self.body_amp_decay = decay_coeff(self.sample_rate, body_decay_ms);

        self.body_tone_filter.set_cutoff(body_tone);
        self.body_tone_filter.set_resonance(body_resonance);

        self.click.set_click_type(click_type_val);
        self.click.set_pitch(click_pitch);
        self.click.set_decay_ms(click_decay);
        self.click.set_filter_freq(click_filter);

        self.sub_osc
            .set_frequency(sub_freq * tuning_factor);
        self.sub_amp_decay = decay_coeff(self.sample_rate, sub_decay_ms);

        // Master EQ (peaking bands at 80, 1000, 8000 Hz)
        self.eq_low
            .set_peaking_eq(80.0, eq_low_db, 0.7, self.sample_rate);
        self.eq_mid
            .set_peaking_eq(1000.0, eq_mid_db, 0.7, self.sample_rate);
        self.eq_high
            .set_peaking_eq(8000.0, eq_high_db, 0.7, self.sample_rate);

        // FX EQ bands: band 1 = low shelf, bands 2-3 = peaking, band 4 = high shelf
        if fx_eq_on {
            self.fx_eq1.set_low_shelf(fx_eq1_freq, fx_eq1_gain, self.sample_rate);
            self.fx_eq2.set_peaking_eq(fx_eq2_freq, fx_eq2_gain, fx_eq2_q, self.sample_rate);
            self.fx_eq3.set_peaking_eq(fx_eq3_freq, fx_eq3_gain, fx_eq3_q, self.sample_rate);
            self.fx_eq4.set_high_shelf(fx_eq4_freq, fx_eq4_gain, self.sample_rate);
        }

        // FX Compressor
        if fx_comp_on {
            let thresh_linear = 10.0_f32.powf(fx_comp_thresh_db / 20.0);
            self.fx_compressor.set_threshold(thresh_linear);
            self.fx_compressor.set_ratio(fx_comp_ratio);
            self.fx_compressor.set_attack_ms(fx_comp_attack);
            self.fx_compressor.set_release_ms(fx_comp_release);
        }

        // FX Distortion
        if fx_dist_on {
            self.fx_distortion.set_type(fx_dist_type);
            self.fx_distortion.set_drive(fx_dist_drive);
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

                    // Trigger body
                    self.body_osc.reset();
                    self.body_pitch_env.trigger();
                    self.body_amp_level = 1.0;

                    // Trigger click
                    if click_on {
                        self.click.trigger();
                    }

                    // Trigger sub
                    if sub_on {
                        self.sub_osc.reset();
                        self.sub_amp_level = 1.0;
                    }
                }

                next_event = context.next_event();
            }

            // ── Body: pitch env → oscillator → distortion (oversampled) → tone filter ──
            let body_freq = self.body_pitch_env.process();
            self.body_osc.set_frequency(body_freq);
            let body_raw = self.body_osc.process();

            // Oversampled distortion to reduce aliasing
            let body_distorted = self
                .body_oversampler
                .process(body_raw, |s| self.body_distortion.process(s));

            // Tone filter (low-pass) on distorted body
            let body_filtered = self.body_tone_filter.process(body_distorted);

            let body_sample = body_filtered * self.body_amp_level * body_vol;
            self.body_amp_level *= self.body_amp_decay;
            if self.body_amp_level < 0.0001 {
                self.body_amp_level = 0.0;
            }

            // ── Click transient ──
            let click_sample = if click_on {
                self.click.process() * click_vol
            } else {
                0.0
            };

            // ── Sub ──
            let sub_sample = if sub_on {
                let s = self.sub_osc.process() * self.sub_amp_level * sub_vol;
                self.sub_amp_level *= self.sub_amp_decay;
                if self.sub_amp_level < 0.0001 {
                    self.sub_amp_level = 0.0;
                }
                s
            } else {
                0.0
            };

            // ── Mix layers (velocity-scaled) ──
            let mut mixed =
                (body_sample + click_sample + sub_sample) * master_vol * self.velocity;

            // ── Master 3-band EQ ──
            mixed = self.eq_low.process(mixed);
            mixed = self.eq_mid.process(mixed);
            mixed = self.eq_high.process(mixed);

            // ── FX: Parametric EQ ──
            if fx_eq_on {
                mixed = self.fx_eq1.process(mixed);
                mixed = self.fx_eq2.process(mixed);
                mixed = self.fx_eq3.process(mixed);
                mixed = self.fx_eq4.process(mixed);
            }

            // ── FX: Compressor ──
            if fx_comp_on {
                mixed = self.fx_compressor.process(mixed);
                // Apply makeup gain
                let makeup_linear = 10.0_f32.powf(fx_comp_makeup / 20.0);
                mixed *= makeup_linear;
            }

            // ── FX: Post Distortion ──
            if fx_dist_on {
                let dry = mixed;
                let wet = self.fx_distortion.process(mixed);
                mixed = dry * (1.0 - fx_dist_mix) + wet * fx_dist_mix;
            }

            // ── Soft limiter ──
            if limiter_on {
                mixed = self.limiter.process(mixed);
            }

            // Write mono kick to stereo output
            left[sample_idx] = mixed;
            right[sample_idx] = mixed;

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
                sub_enabled: sub_on,
                sub_frequency: sub_freq,
                sub_volume: sub_vol,
                sub_decay: sub_decay_ms,
                master_volume: master_vol,
                master_tuning,
                master_octave: master_octave as i32,
                master_limiter: limiter_on,
                master_low: eq_low_db,
                master_mid: eq_mid_db,
                master_high: eq_high_db,
                fx_eq_enabled: fx_eq_on,
                fx_eq1_freq, fx_eq1_gain, fx_eq1_q,
                fx_eq2_freq, fx_eq2_gain, fx_eq2_q,
                fx_eq3_freq, fx_eq3_gain, fx_eq3_q,
                fx_eq4_freq, fx_eq4_gain, fx_eq4_q,
                fx_comp_enabled: fx_comp_on,
                fx_comp_threshold: fx_comp_thresh_db,
                fx_comp_ratio,
                fx_comp_attack,
                fx_comp_release,
                fx_comp_makeup,
                fx_dist_enabled: fx_dist_on,
                fx_dist_type: fx_dist_type as i32,
                fx_dist_drive,
                fx_dist_mix,
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
