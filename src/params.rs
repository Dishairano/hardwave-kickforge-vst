//! DAW-exposed parameters for the KickForge kick synthesizer.
//!
//! Every field has a corresponding nih-plug parameter so the DAW can
//! automate / save / recall them. Enums use nih_plug's EnumParam.

use nih_plug::prelude::*;

use crate::dsp::click::ClickType;
use crate::dsp::distortion::DistortionType;
use crate::dsp::oscillator::Waveform;
use crate::dsp::pitch_envelope::PitchCurve;

#[derive(Params)]
pub struct KickForgeParams {
    // ── Click layer ─────────────────────────────────────────────────────────
    #[id = "click_enabled"]
    pub click_enabled: BoolParam,

    #[id = "click_type"]
    pub click_type: EnumParam<ClickType>,

    #[id = "click_volume"]
    pub click_volume: FloatParam,

    #[id = "click_pitch"]
    pub click_pitch: FloatParam,

    #[id = "click_decay"]
    pub click_decay: FloatParam,

    #[id = "click_filter_freq"]
    pub click_filter_freq: FloatParam,

    // ── Body layer ──────────────────────────────────────────────────────────
    #[id = "body_pitch_start"]
    pub body_pitch_start: FloatParam,

    #[id = "body_pitch_end"]
    pub body_pitch_end: FloatParam,

    #[id = "body_pitch_decay"]
    pub body_pitch_decay: FloatParam,

    #[id = "body_pitch_curve"]
    pub body_pitch_curve: EnumParam<PitchCurve>,

    #[id = "body_waveform"]
    pub body_waveform: EnumParam<Waveform>,

    #[id = "body_drive"]
    pub body_drive: FloatParam,

    #[id = "body_distortion_type"]
    pub body_distortion_type: EnumParam<DistortionType>,

    #[id = "body_decay"]
    pub body_decay: FloatParam,

    #[id = "body_volume"]
    pub body_volume: FloatParam,

    #[id = "body_tone"]
    pub body_tone: FloatParam,

    #[id = "body_resonance"]
    pub body_resonance: FloatParam,

    // ── Sub layer ───────────────────────────────────────────────────────────
    #[id = "sub_enabled"]
    pub sub_enabled: BoolParam,

    #[id = "sub_frequency"]
    pub sub_frequency: FloatParam,

    #[id = "sub_volume"]
    pub sub_volume: FloatParam,

    #[id = "sub_decay"]
    pub sub_decay: FloatParam,

    // ── Master ──────────────────────────────────────────────────────────────
    #[id = "master_volume"]
    pub master_volume: FloatParam,

    #[id = "master_tuning"]
    pub master_tuning: FloatParam,

    #[id = "master_limiter"]
    pub master_limiter: BoolParam,

    #[id = "master_low"]
    pub master_low: FloatParam,

    #[id = "master_mid"]
    pub master_mid: FloatParam,

    #[id = "master_high"]
    pub master_high: FloatParam,
}

impl Default for KickForgeParams {
    fn default() -> Self {
        Self {
            // ── Click ───────────────────────────────────────────────────────
            click_enabled: BoolParam::new("Click On", true),

            click_type: EnumParam::new("Click Type", ClickType::Noise),

            click_volume: FloatParam::new(
                "Click Volume",
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            click_pitch: FloatParam::new(
                "Click Pitch",
                4000.0,
                FloatRange::Skewed {
                    min: 1000.0,
                    max: 12000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            click_decay: FloatParam::new(
                "Click Decay",
                5.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 50.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" ms"),

            click_filter_freq: FloatParam::new(
                "Click Filter",
                8000.0,
                FloatRange::Skewed {
                    min: 500.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            // ── Body ────────────────────────────────────────────────────────
            body_pitch_start: FloatParam::new(
                "Body Pitch Start",
                800.0,
                FloatRange::Skewed {
                    min: 200.0,
                    max: 5000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            body_pitch_end: FloatParam::new(
                "Body Pitch End",
                50.0,
                FloatRange::Skewed {
                    min: 30.0,
                    max: 100.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz"),

            body_pitch_decay: FloatParam::new(
                "Body Pitch Decay",
                150.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" ms"),

            body_pitch_curve: EnumParam::new("Pitch Curve", PitchCurve::Exponential),

            body_waveform: EnumParam::new("Body Waveform", Waveform::Sine),

            body_drive: FloatParam::new(
                "Body Drive",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            body_distortion_type: EnumParam::new("Distortion Type", DistortionType::Tanh),

            body_decay: FloatParam::new(
                "Body Decay",
                500.0,
                FloatRange::Skewed {
                    min: 50.0,
                    max: 5000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" ms"),

            body_volume: FloatParam::new(
                "Body Volume",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            body_tone: FloatParam::new(
                "Body Tone",
                8000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            body_resonance: FloatParam::new(
                "Body Resonance",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // ── Sub ─────────────────────────────────────────────────────────
            sub_enabled: BoolParam::new("Sub On", true),

            sub_frequency: FloatParam::new(
                "Sub Frequency",
                50.0,
                FloatRange::Skewed {
                    min: 30.0,
                    max: 80.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz"),

            sub_volume: FloatParam::new(
                "Sub Volume",
                0.6,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            sub_decay: FloatParam::new(
                "Sub Decay",
                300.0,
                FloatRange::Skewed {
                    min: 50.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" ms"),

            // ── Master ──────────────────────────────────────────────────────
            master_volume: FloatParam::new(
                "Master Volume",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            master_tuning: FloatParam::new(
                "Master Tuning",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" st"),

            master_limiter: BoolParam::new("Limiter", true),

            master_low: FloatParam::new(
                "Low",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),

            master_mid: FloatParam::new(
                "Mid",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),

            master_high: FloatParam::new(
                "High",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_unit(" dB"),
        }
    }
}
