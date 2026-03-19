//! DAW-exposed parameters for the KickForge kick synthesizer.

use nih_plug::prelude::*;

use crate::dsp::click::ClickType;
use crate::dsp::distortion::DistortionType;
use crate::dsp::noise::NoiseType;
use crate::dsp::oscillator::Waveform;
use crate::dsp::pitch_envelope::PitchCurve;

/// Number of modular FX slots in the rack.
pub const NUM_FX_SLOTS: usize = 8;

/// FX slot types.
/// 0 = Empty, 1 = EQ, 2 = Compressor, 3 = Distortion, 4 = Transient
pub const FX_EMPTY: i32 = 0;
pub const FX_EQ: i32 = 1;
pub const FX_COMP: i32 = 2;
pub const FX_DIST: i32 = 3;
pub const FX_TRANS: i32 = 4;

/// Parameters for a single FX slot.
/// Uses 6 generic normalized (0-1) params whose meaning depends on slot type.
#[derive(Params)]
pub struct FxSlotParams {
    /// 0=Empty, 1=EQ, 2=Comp, 3=Dist, 4=Transient
    #[id = "type"]
    pub slot_type: IntParam,

    #[id = "on"]
    pub enabled: BoolParam,

    #[id = "p1"]
    pub p1: FloatParam,
    #[id = "p2"]
    pub p2: FloatParam,
    #[id = "p3"]
    pub p3: FloatParam,
    #[id = "p4"]
    pub p4: FloatParam,
    #[id = "p5"]
    pub p5: FloatParam,
    #[id = "p6"]
    pub p6: FloatParam,
}

impl Default for FxSlotParams {
    fn default() -> Self {
        Self {
            slot_type: IntParam::new("Type", 0, IntRange::Linear { min: 0, max: 4 }),
            enabled: BoolParam::new("On", true),
            p1: FloatParam::new("P1", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            p2: FloatParam::new("P2", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            p3: FloatParam::new("P3", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            p4: FloatParam::new("P4", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            p5: FloatParam::new("P5", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            p6: FloatParam::new("P6", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

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

    /// Oscillator self-modulation (FM feedback). Thickens the body tone
    /// with additional harmonics before distortion.
    #[id = "body_feedback"]
    pub body_feedback: FloatParam,

    /// Hold time in ms — the body stays at full level for this long before
    /// decay starts. Creates the "punch window" that defines a kick vs a laser.
    #[id = "body_hold"]
    pub body_hold: FloatParam,

    /// Frequency (Hz) below which the signal is kept clean during distortion.
    /// Above this frequency, the body is distorted normally. The clean sub
    /// is mixed back in underneath, keeping the low-end tight.
    #[id = "body_split_freq"]
    pub body_split_freq: FloatParam,

    // ── Sub layer (legacy — now controls the clean sub from split-band) ────
    /// When enabled, the clean sub from split-band distortion is mixed in.
    #[id = "sub_enabled"]
    pub sub_enabled: BoolParam,

    /// Controls the level of the clean sub component.
    #[id = "sub_volume"]
    pub sub_volume: FloatParam,

    /// Legacy param kept for project compat — no longer drives a separate osc.
    #[id = "sub_frequency"]
    pub sub_frequency: FloatParam,

    /// Legacy param kept for project compat.
    #[id = "sub_decay"]
    pub sub_decay: FloatParam,

    // ── Noise layer ─────────────────────────────────────────────────────────
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

    // ── Layer solo ──────────────────────────────────────────────────────────
    #[id = "click_solo"]
    pub click_solo: BoolParam,

    #[id = "body_solo"]
    pub body_solo: BoolParam,

    #[id = "sub_solo"]
    pub sub_solo: BoolParam,

    #[id = "noise_solo"]
    pub noise_solo: BoolParam,

    // ── Velocity mapping ────────────────────────────────────────────────────
    #[id = "vel_to_decay"]
    pub vel_to_decay: FloatParam,

    #[id = "vel_to_pitch"]
    pub vel_to_pitch: FloatParam,

    #[id = "vel_to_drive"]
    pub vel_to_drive: FloatParam,

    #[id = "vel_to_click"]
    pub vel_to_click: FloatParam,

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

    // ── Modular FX Rack (8 slots) ───────────────────────────────────────────
    #[nested(array, group = "FX Slot")]
    pub fx_slots: [FxSlotParams; NUM_FX_SLOTS],
}

impl Default for KickForgeParams {
    fn default() -> Self {
        Self {
            // ── Click ───────────────────────────────────────────────────────
            click_enabled: BoolParam::new("Click On", true),
            click_type: EnumParam::new("Click Type", ClickType::Noise),
            click_volume: FloatParam::new(
                "Click Volume", 0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            click_pitch: FloatParam::new(
                "Click Pitch", 4000.0,
                FloatRange::Skewed { min: 1000.0, max: 12000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            click_decay: FloatParam::new(
                "Click Decay", 5.0,
                FloatRange::Skewed { min: 1.0, max: 50.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" ms"),
            click_filter_freq: FloatParam::new(
                "Click Filter", 8000.0,
                FloatRange::Skewed { min: 500.0, max: 20000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            // ── Body ────────────────────────────────────────────────────────
            body_pitch_start: FloatParam::new(
                "Body Pitch Start", 800.0,
                FloatRange::Skewed { min: 200.0, max: 5000.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            body_pitch_end: FloatParam::new(
                "Body Pitch End", 50.0,
                FloatRange::Skewed { min: 30.0, max: 100.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz"),
            body_pitch_decay: FloatParam::new(
                "Body Pitch Decay", 150.0,
                FloatRange::Skewed { min: 10.0, max: 1000.0, factor: FloatRange::skew_factor(-1.5) },
            ).with_unit(" ms"),
            body_pitch_curve: EnumParam::new("Pitch Curve", PitchCurve::Exponential),
            body_waveform: EnumParam::new("Body Waveform", Waveform::Sine),
            body_drive: FloatParam::new(
                "Body Drive", 0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            body_distortion_type: EnumParam::new("Distortion Type", DistortionType::Tanh),
            body_decay: FloatParam::new(
                "Body Decay", 500.0,
                FloatRange::Skewed { min: 50.0, max: 5000.0, factor: FloatRange::skew_factor(-1.5) },
            ).with_unit(" ms"),
            body_volume: FloatParam::new(
                "Body Volume", 1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            body_tone: FloatParam::new(
                "Body Tone", 8000.0,
                FloatRange::Skewed { min: 20.0, max: 20000.0, factor: FloatRange::skew_factor(-2.0) },
            ).with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            body_resonance: FloatParam::new(
                "Body Resonance", 0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            body_feedback: FloatParam::new(
                "Body Feedback", 0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            body_hold: FloatParam::new(
                "Body Hold", 10.0,
                FloatRange::Skewed { min: 0.0, max: 80.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" ms"),
            body_split_freq: FloatParam::new(
                "Sub Split", 120.0,
                FloatRange::Skewed { min: 40.0, max: 300.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz"),

            // ── Sub ─────────────────────────────────────────────────────────
            sub_enabled: BoolParam::new("Sub On", true),
            sub_frequency: FloatParam::new(
                "Sub Frequency", 50.0,
                FloatRange::Skewed { min: 30.0, max: 80.0, factor: FloatRange::skew_factor(-1.0) },
            ).with_unit(" Hz"),
            sub_volume: FloatParam::new(
                "Sub Volume", 0.6,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            sub_decay: FloatParam::new(
                "Sub Decay", 300.0,
                FloatRange::Skewed { min: 50.0, max: 2000.0, factor: FloatRange::skew_factor(-1.5) },
            ).with_unit(" ms"),

            // ── Noise ───────────────────────────────────────────────────────
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

            // ── Layer solo ──────────────────────────────────────────────────
            click_solo: BoolParam::new("Click Solo", false),
            body_solo: BoolParam::new("Body Solo", false),
            sub_solo: BoolParam::new("Sub Solo", false),
            noise_solo: BoolParam::new("Noise Solo", false),

            // ── Velocity mapping ────────────────────────────────────────────
            vel_to_decay: FloatParam::new(
                "Vel→Decay", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 },
            ),
            vel_to_pitch: FloatParam::new(
                "Vel→Pitch", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 },
            ),
            vel_to_drive: FloatParam::new(
                "Vel→Drive", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 },
            ),
            vel_to_click: FloatParam::new(
                "Vel→Click", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 },
            ),

            // ── Master ──────────────────────────────────────────────────────
            master_volume: FloatParam::new(
                "Master Volume", 0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ).with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            master_tuning: FloatParam::new(
                "Master Tuning", 0.0,
                FloatRange::Linear { min: -12.0, max: 12.0 },
            ).with_unit(" st"),
            master_octave: IntParam::new("Octave", 0, IntRange::Linear { min: -4, max: 4 }),
            master_limiter: BoolParam::new("Limiter", true),
            master_low: FloatParam::new(
                "Low", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 },
            ).with_unit(" dB"),
            master_mid: FloatParam::new(
                "Mid", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 },
            ).with_unit(" dB"),
            master_high: FloatParam::new(
                "High", 0.0, FloatRange::Linear { min: -12.0, max: 12.0 },
            ).with_unit(" dB"),

            // ── FX Rack (8 slots, all empty by default) ─────────────────────
            fx_slots: Default::default(),
        }
    }
}
