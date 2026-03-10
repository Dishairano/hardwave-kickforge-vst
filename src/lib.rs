//! KickForge — hardstyle & hardcore kick synthesizer VST3/CLAP plugin by Hardwave Studios.
//!
//! Architecture:
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
mod protocol;

use dsp::click::Click;
use dsp::distortion::Distortion;
use dsp::oscillator::Oscillator;
use dsp::pitch_envelope::PitchEnvelope;
use params::KickForgeParams;
use protocol::KickForgePacket;

/// How often we send param state to the editor (every N process calls).
const EDITOR_UPDATE_INTERVAL: u32 = 512;

pub struct HardwaveKickForge {
    params: Arc<KickForgeParams>,

    // DSP state
    body_osc: Oscillator,
    body_pitch_env: PitchEnvelope,
    body_distortion: Distortion,
    /// Body amplitude envelope level (simple exponential decay).
    body_amp_level: f32,
    body_amp_decay: f32,

    click: Click,

    sub_osc: Oscillator,
    /// Sub amplitude envelope level.
    sub_amp_level: f32,
    sub_amp_decay: f32,

    sample_rate: f32,

    // Plugin -> Editor: latest param state
    editor_packet_tx: Sender<KickForgePacket>,
    editor_packet_rx: Arc<Mutex<Receiver<KickForgePacket>>>,

    // Counter for throttled editor updates
    update_counter: u32,
}

/// Calculate a per-sample exponential decay coefficient from decay time in ms.
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
            body_amp_level: 0.0,
            body_amp_decay: decay_coeff(sr, 500.0),
            click: Click::new(sr),
            sub_osc: Oscillator::new(sr),
            sub_amp_level: 0.0,
            sub_amp_decay: decay_coeff(sr, 300.0),
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

    // Instrument: no audio input, stereo output
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

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
        self.click.set_sample_rate(self.sample_rate);
        self.sub_osc.set_sample_rate(self.sample_rate);

        true
    }

    fn reset(&mut self) {
        self.body_osc.reset();
        self.body_pitch_env.reset();
        self.body_distortion.reset();
        self.body_amp_level = 0.0;
        self.click.reset();
        self.sub_osc.reset();
        self.sub_amp_level = 0.0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Read current param values
        let click_on = self.params.click_enabled.value();
        let click_vol = self.params.click_volume.value();
        let click_pitch = self.params.click_pitch.value();
        let click_decay = self.params.click_decay.value();

        let body_pitch_start = self.params.body_pitch_start.value();
        let body_pitch_end = self.params.body_pitch_end.value();
        let body_pitch_decay = self.params.body_pitch_decay.value();
        let body_drive = self.params.body_drive.value();
        let body_vol = self.params.body_volume.value();

        let sub_on = self.params.sub_enabled.value();
        let sub_freq = self.params.sub_frequency.value();
        let sub_vol = self.params.sub_volume.value();
        let sub_decay_ms = self.params.sub_decay.value();

        let master_vol = self.params.master_volume.value();
        let master_tuning = self.params.master_tuning.value();

        // Update DSP parameters
        self.body_pitch_env.set_start_freq(body_pitch_start * 2.0_f32.powf(master_tuning / 12.0));
        self.body_pitch_env.set_end_freq(body_pitch_end * 2.0_f32.powf(master_tuning / 12.0));
        self.body_pitch_env.set_decay_ms(body_pitch_decay);
        self.body_distortion.set_drive(body_drive);
        // Body amp decay: use pitch decay * 3 as a rough body length
        self.body_amp_decay = decay_coeff(self.sample_rate, body_pitch_decay * 3.0);

        self.click.set_pitch(click_pitch);
        self.click.set_decay_ms(click_decay);

        self.sub_osc.set_frequency(sub_freq * 2.0_f32.powf(master_tuning / 12.0));
        self.sub_amp_decay = decay_coeff(self.sample_rate, sub_decay_ms);

        let num_samples = buffer.samples();

        // Process MIDI events and audio sample-by-sample
        let mut next_event = context.next_event();
        let mut sample_idx = 0;

        let output = buffer.as_slice();
        // We have stereo output (2 channels)
        let (left, right_and_rest) = output.split_first_mut().unwrap();
        let right = &mut right_and_rest[0];

        while sample_idx < num_samples {
            // Handle MIDI events at or before this sample
            while let Some(event) = next_event {
                if event.timing() > sample_idx as u32 {
                    break;
                }

                if let NoteEvent::NoteOn { .. } = event {
                    // Trigger all layers
                    self.body_osc.reset();
                    self.body_pitch_env.trigger();
                    self.body_amp_level = 1.0;

                    if click_on {
                        self.click.trigger();
                    }

                    if sub_on {
                        self.sub_osc.reset();
                        self.sub_amp_level = 1.0;
                    }
                }

                next_event = context.next_event();
            }

            // --- Generate audio ---

            // Body: pitch envelope -> oscillator -> distortion
            let body_freq = self.body_pitch_env.process();
            self.body_osc.set_frequency(body_freq);
            let mut body_sample = self.body_osc.process();
            body_sample = self.body_distortion.process(body_sample);
            body_sample *= self.body_amp_level * body_vol;
            self.body_amp_level *= self.body_amp_decay;
            if self.body_amp_level < 0.0001 {
                self.body_amp_level = 0.0;
            }

            // Click transient
            let click_sample = if click_on {
                self.click.process() * click_vol
            } else {
                0.0
            };

            // Sub
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

            // Mix layers
            let mixed = (body_sample + click_sample + sub_sample) * master_vol;

            // Write to both channels (mono kick -> stereo)
            left[sample_idx] = mixed;
            right[sample_idx] = mixed;

            sample_idx += 1;
        }

        // Throttled editor update
        self.update_counter += num_samples as u32;
        if self.update_counter >= EDITOR_UPDATE_INTERVAL {
            self.update_counter = 0;
            let packet = KickForgePacket {
                click_enabled: click_on,
                click_volume: click_vol,
                click_pitch,
                click_decay,
                body_pitch_start,
                body_pitch_end,
                body_pitch_decay,
                body_drive,
                body_volume: body_vol,
                sub_enabled: sub_on,
                sub_frequency: sub_freq,
                sub_volume: sub_vol,
                sub_decay: sub_decay_ms,
                master_volume: master_vol,
                master_tuning,
            };
            let _ = self.editor_packet_tx.try_send(packet);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for HardwaveKickForge {
    const CLAP_ID: &'static str = "com.hardwavestudios.kickforge";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Hardstyle & hardcore kick synthesizer");
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
