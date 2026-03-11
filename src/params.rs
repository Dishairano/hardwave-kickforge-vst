//! DAW-exposed parameters for the KickForge kick synthesizer.
//!
//! Every field has a corresponding nih-plug parameter so the DAW can
//! automate / save / recall them. Enums use nih_plug's EnumParam.

use nih_plug::prelude::*;

use crate::dsp::click::ClickType;
use crate::dsp::distortion::DistortionType;
use crate::dsp::noise::NoiseType;
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

    // ── Noise layer ──────────────────────────────────────────────────────────
    #[id = "noise_enabled"]
    pub noise_enabled: BoolParam,

    #[id = "noise_type"]
    pub noise_type: EnumParam<NoiseType>,

    #[id = "noise_volume"]
    pub noise_volume: FloatParam,

    #[id = "noise_decay"]
    pub noise_decay: FloatParam,

    #[id = "noise_filter_freq"]
    pub noise_filter_freq: FloatParam,

    // ── Layer solo ────────────────────────────────────────────────────────────
    #[id = "click_solo"]
    pub click_solo: BoolParam,

    #[id = "body_solo"]
    pub body_solo: BoolParam,

    #[id = "sub_solo"]
    pub sub_solo: BoolParam,

    #[id = "noise_solo"]
    pub noise_solo: BoolParam,

    // ── Velocity mapping ──────────────────────────────────────────────────────
    #[id = "vel_to_decay"]
    pub vel_to_decay: FloatParam,

    #[id = "vel_to_pitch"]
    pub vel_to_pitch: FloatParam,

    #[id = "vel_to_drive"]
    pub vel_to_drive: FloatParam,

    #[id = "vel_to_click"]
    pub vel_to_click: FloatParam,

    // ── FX: Transient Shaper ──────────────────────────────────────────────────
    #[id = "fx_trans_enabled"]
    pub fx_trans_enabled: BoolParam,

    #[id = "fx_trans_attack"]
    pub fx_trans_attack: FloatParam,

    #[id = "fx_trans_sustain"]
    pub fx_trans_sustain: FloatParam,

    // ── Master ──────────────────────────────────────────────────────────────
    #[id = "master_volume"]
    pub master_volume: FloatParam,

    #[id = "master_tuning"]
    pub master_tuning: FloatParam,

    #[id = "master_octave"]
    pub master_octave: IntParam,

    #[id = "master_limiter"]
    pub master_limiter: BoolParam,

    #[id = "master_low"]
    pub master_low: FloatParam,

    #[id = "master_mid"]
    pub master_mid: FloatParam,

    #[id = "master_high"]
    pub master_high: FloatParam,

    // ── FX: Parametric EQ (4 bands) ───────────────────────────────────────
    #[id = "fx_eq_enabled"]
    pub fx_eq_enabled: BoolParam,

    #[id = "fx_eq1_freq"]
    pub fx_eq1_freq: FloatParam,
    #[id = "fx_eq1_gain"]
    pub fx_eq1_gain: FloatParam,
    #[id = "fx_eq1_q"]
    pub fx_eq1_q: FloatParam,

    #[id = "fx_eq2_freq"]
    pub fx_eq2_freq: FloatParam,
    #[id = "fx_eq2_gain"]
    pub fx_eq2_gain: FloatParam,
    #[id = "fx_eq2_q"]
    pub fx_eq2_q: FloatParam,

    #[id = "fx_eq3_freq"]
    pub fx_eq3_freq: FloatParam,
    #[id = "fx_eq3_gain"]
    pub fx_eq3_gain: FloatParam,
    #[id = "fx_eq3_q"]
    pub fx_eq3_q: FloatParam,

    #[id = "fx_eq4_freq"]
    pub fx_eq4_freq: FloatParam,
    #[id = "fx_eq4_gain"]
    pub fx_eq4_gain: FloatParam,
    #[id = "fx_eq4_q"]
    pub fx_eq4_q: FloatParam,

    // ── FX: Compressor ────────────────────────────────────────────────────
    #[id = "fx_comp_enabled"]
    pub fx_comp_enabled: BoolParam,

    #[id = "fx_comp_threshold"]
    pub fx_comp_threshold: FloatParam,

    #[id = "fx_comp_ratio"]
    pub fx_comp_ratio: FloatParam,

    #[id = "fx_comp_attack"]
    pub fx_comp_attack: FloatParam,

    #[id = "fx_comp_release"]
    pub fx_comp_release: FloatParam,

    #[id = "fx_comp_makeup"]
    pub fx_comp_makeup: FloatParam,

    // ── FX: Post Distortion ───────────────────────────────────────────────
    #[id = "fx_dist_enabled"]
    pub fx_dist_enabled: BoolParam,

    #[id = "fx_dist_type"]
    pub fx_dist_type: EnumParam<DistortionType>,

    #[id = "fx_dist_drive"]
    pub fx_dist_drive: FloatParam,

    #[id = "fx_dist_mix"]
    pub fx_dist_mix: FloatParam,
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

            // ── Noise layer ───────────────────────────────────────────────
            noise_enabled: BoolParam::new("Noise On", false),
            noise_type: EnumParam::new("Noise Type", NoiseType::White),
            noise_volume: FloatParam::new(
                "Noise Volume", 0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            noise_decay: FloatParam::new(
                "Noise Decay", 100.0,
                FloatRange::Skewed { min: 10.0, max: 2000.0, factor: FloatRange::skew_factor(-1.5) },
            ).with_unit(" ms"),
            noise_filter_freq: FloatParam::new(
                "Noise Filter", 5000.0,
                FloatRange::Skewed { min: 200.0, max: 20000.0, factor: FloatRange::skew_factor(-2.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            // ── Layer solo ───────────────────────────────────────────────────
            click_solo: BoolParam::new("Click Solo", false),
            body_solo: BoolParam::new("Body Solo", false),
            sub_solo: BoolParam::new("Sub Solo", false),
            noise_solo: BoolParam::new("Noise Solo", false),

            // ── Velocity mapping ─────────────────────────────────────────────
            vel_to_decay: FloatParam::new(
                "Vel→Decay", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            vel_to_pitch: FloatParam::new(
                "Vel→Pitch", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            vel_to_drive: FloatParam::new(
                "Vel→Drive", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            vel_to_click: FloatParam::new(
                "Vel→Click", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // ── FX: Transient Shaper ─────────────────────────────────────────
            fx_trans_enabled: BoolParam::new("FX Transient On", false),
            fx_trans_attack: FloatParam::new(
                "Trans Attack", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ),
            fx_trans_sustain: FloatParam::new(
                "Trans Sustain", 0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            ),

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

            master_octave: IntParam::new("Octave", 0, IntRange::Linear { min: -4, max: 4 }),

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

            // ── FX: Parametric EQ ──────────────────────────────────────────
            fx_eq_enabled: BoolParam::new("FX EQ On", false),

            // Band 1: Low Shelf @ 80 Hz
            fx_eq1_freq: FloatParam::new(
                "EQ1 Freq",
                80.0,
                FloatRange::Skewed { min: 20.0, max: 500.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz"),
            fx_eq1_gain: FloatParam::new(
                "EQ1 Gain",
                0.0,
                FloatRange::Linear { min: -18.0, max: 18.0 },
            ).with_unit(" dB"),
            fx_eq1_q: FloatParam::new(
                "EQ1 Q",
                0.7,
                FloatRange::Skewed { min: 0.1, max: 10.0, factor: FloatRange::skew_factor(-1.0) },
            ),

            // Band 2: Peak @ 500 Hz
            fx_eq2_freq: FloatParam::new(
                "EQ2 Freq",
                500.0,
                FloatRange::Skewed { min: 100.0, max: 4000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz"),
            fx_eq2_gain: FloatParam::new(
                "EQ2 Gain",
                0.0,
                FloatRange::Linear { min: -18.0, max: 18.0 },
            ).with_unit(" dB"),
            fx_eq2_q: FloatParam::new(
                "EQ2 Q",
                0.7,
                FloatRange::Skewed { min: 0.1, max: 10.0, factor: FloatRange::skew_factor(-1.0) },
            ),

            // Band 3: Peak @ 3 kHz
            fx_eq3_freq: FloatParam::new(
                "EQ3 Freq",
                3000.0,
                FloatRange::Skewed { min: 500.0, max: 12000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            fx_eq3_gain: FloatParam::new(
                "EQ3 Gain",
                0.0,
                FloatRange::Linear { min: -18.0, max: 18.0 },
            ).with_unit(" dB"),
            fx_eq3_q: FloatParam::new(
                "EQ3 Q",
                0.7,
                FloatRange::Skewed { min: 0.1, max: 10.0, factor: FloatRange::skew_factor(-1.0) },
            ),

            // Band 4: High Shelf @ 10 kHz
            fx_eq4_freq: FloatParam::new(
                "EQ4 Freq",
                10000.0,
                FloatRange::Skewed { min: 2000.0, max: 20000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            fx_eq4_gain: FloatParam::new(
                "EQ4 Gain",
                0.0,
                FloatRange::Linear { min: -18.0, max: 18.0 },
            ).with_unit(" dB"),
            fx_eq4_q: FloatParam::new(
                "EQ4 Q",
                0.7,
                FloatRange::Skewed { min: 0.1, max: 10.0, factor: FloatRange::skew_factor(-1.0) },
            ),

            // ── FX: Compressor ─────────────────────────────────────────────
            fx_comp_enabled: BoolParam::new("FX Comp On", false),

            fx_comp_threshold: FloatParam::new(
                "Comp Threshold",
                -12.0,
                FloatRange::Linear { min: -40.0, max: 0.0 },
            ).with_unit(" dB"),

            fx_comp_ratio: FloatParam::new(
                "Comp Ratio",
                4.0,
                FloatRange::Skewed { min: 1.0, max: 20.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(":1"),

            fx_comp_attack: FloatParam::new(
                "Comp Attack",
                5.0,
                FloatRange::Skewed { min: 0.1, max: 100.0, factor: FloatRange::skew_factor(-1.5) },
            ).with_unit(" ms"),

            fx_comp_release: FloatParam::new(
                "Comp Release",
                50.0,
                FloatRange::Skewed { min: 10.0, max: 500.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" ms"),

            fx_comp_makeup: FloatParam::new(
                "Comp Makeup",
                0.0,
                FloatRange::Linear { min: 0.0, max: 24.0 },
            ).with_unit(" dB"),

            // ── FX: Post Distortion ────────────────────────────────────────
            fx_dist_enabled: BoolParam::new("FX Dist On", false),

            fx_dist_type: EnumParam::new("FX Dist Type", DistortionType::Tanh),

            fx_dist_drive: FloatParam::new(
                "FX Dist Drive",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            fx_dist_mix: FloatParam::new(
                "FX Dist Mix",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}
