//! DAW-exposed parameters for the KickForge kick synthesizer.
//!
//! Every field has a corresponding nih-plug parameter so the DAW can
//! automate / save / recall them.

use nih_plug::prelude::*;

#[derive(Params)]
pub struct KickForgeParams {
    // ── Click layer ─────────────────────────────────────────────────────────
    #[id = "click_enabled"]
    pub click_enabled: BoolParam,

    #[id = "click_volume"]
    pub click_volume: FloatParam,

    #[id = "click_pitch"]
    pub click_pitch: FloatParam,

    #[id = "click_decay"]
    pub click_decay: FloatParam,

    // ── Body layer ──────────────────────────────────────────────────────────
    #[id = "body_pitch_start"]
    pub body_pitch_start: FloatParam,

    #[id = "body_pitch_end"]
    pub body_pitch_end: FloatParam,

    #[id = "body_pitch_decay"]
    pub body_pitch_decay: FloatParam,

    #[id = "body_drive"]
    pub body_drive: FloatParam,

    #[id = "body_volume"]
    pub body_volume: FloatParam,

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
}

impl Default for KickForgeParams {
    fn default() -> Self {
        Self {
            // Click
            click_enabled: BoolParam::new("Click On", true),
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

            // Body
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

            body_drive: FloatParam::new(
                "Body Drive",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            body_volume: FloatParam::new(
                "Body Volume",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            // Sub
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

            // Master
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
                FloatRange::Linear { min: -12.0, max: 12.0 },
            )
            .with_unit(" st"),
        }
    }
}
