//! Factory presets for KickForge.
//!
//! Each preset is a collection of param_id -> f64 value pairs.
//! Presets are applied via the WebView UI which calls setParam for each value.
//! This module is NOT used in the audio thread.

use std::collections::HashMap;

/// A factory preset: name + parameter values.
pub struct Preset {
    pub name: &'static str,
    pub values: HashMap<String, f64>,
}

/// Build a preset from a list of (key, value) tuples.
fn make_preset(name: &'static str, params: &[(&str, f64)]) -> Preset {
    let mut values = HashMap::new();
    for &(key, val) in params {
        values.insert(key.to_string(), val);
    }
    Preset { name, values }
}

/// Return all factory presets.
pub fn factory_presets() -> Vec<Preset> {
    vec![
        // ── Init: clean starting point ──────────────────────────────────────
        make_preset("Init", &[
            ("click_enabled", 1.0),
            ("click_type", 0.0),        // Noise
            ("click_volume", 0.5),
            ("click_pitch", 4000.0),
            ("click_decay", 5.0),
            ("click_filter_freq", 8000.0),
            ("body_pitch_start", 500.0),
            ("body_pitch_end", 50.0),
            ("body_pitch_decay", 100.0),
            ("body_pitch_curve", 0.0),   // Exponential
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.2),
            ("body_distortion_type", 0.0), // Tanh
            ("body_decay", 300.0),
            ("body_volume", 0.8),
            ("body_tone", 12000.0),
            ("body_resonance", 0.0),
            ("body_feedback", 0.0),
            ("body_hold", 8.0),
            ("body_split_freq", 120.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.8),
            ("master_volume", 0.8),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 0.0),
            ("master_mid", 0.0),
            ("master_high", 0.0),
        ]),

        // ── Classic Hardstyle: punchy body, clean sub, tight tail ───────────
        make_preset("Classic Hardstyle", &[
            ("click_enabled", 1.0),
            ("click_type", 2.0),        // Punch
            ("click_volume", 0.7),
            ("click_pitch", 4500.0),
            ("click_decay", 5.0),
            ("click_filter_freq", 6000.0),
            ("body_pitch_start", 700.0),
            ("body_pitch_end", 55.0),
            ("body_pitch_decay", 140.0),
            ("body_pitch_curve", 4.0),   // Punch curve
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.45),
            ("body_distortion_type", 0.0), // Tanh — warm saturation
            ("body_decay", 500.0),
            ("body_volume", 1.0),
            ("body_tone", 5000.0),
            ("body_resonance", 0.1),
            ("body_feedback", 0.15),     // Slight FM thickens the body
            ("body_hold", 15.0),         // 15ms punch window
            ("body_split_freq", 130.0),  // Keep everything below 130Hz clean
            ("sub_enabled", 1.0),
            ("sub_volume", 0.9),
            ("master_volume", 0.85),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 3.0),
            ("master_mid", -1.0),
            ("master_high", 0.0),
        ]),

        // ── Raw Hardstyle: heavier distortion, more feedback, longer body ──
        make_preset("Raw Hardstyle", &[
            ("click_enabled", 1.0),
            ("click_type", 2.0),        // Punch
            ("click_volume", 0.75),
            ("click_pitch", 5500.0),
            ("click_decay", 4.0),
            ("click_filter_freq", 8000.0),
            ("body_pitch_start", 1000.0),
            ("body_pitch_end", 50.0),
            ("body_pitch_decay", 200.0),
            ("body_pitch_curve", 4.0),   // Punch
            ("body_waveform", 0.0),      // Sine — FM feedback adds harmonics
            ("body_drive", 0.65),
            ("body_distortion_type", 0.0), // Tanh
            ("body_decay", 800.0),
            ("body_volume", 1.0),
            ("body_tone", 4000.0),
            ("body_resonance", 0.2),
            ("body_feedback", 0.35),     // More FM = more aggressive body
            ("body_hold", 12.0),
            ("body_split_freq", 110.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.85),
            ("master_volume", 0.85),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 4.0),
            ("master_mid", 1.0),
            ("master_high", -1.0),
        ]),

        // ── Hardcore / Gabber: fast sweep, hard distortion, short & punchy ──
        make_preset("Hardcore/Gabber", &[
            ("click_enabled", 1.0),
            ("click_type", 0.0),        // Noise
            ("click_volume", 0.85),
            ("click_pitch", 7000.0),
            ("click_decay", 3.0),
            ("click_filter_freq", 10000.0),
            ("body_pitch_start", 1500.0),
            ("body_pitch_end", 48.0),
            ("body_pitch_decay", 50.0),
            ("body_pitch_curve", 0.0),   // Exponential — fast
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.7),
            ("body_distortion_type", 1.0), // HardClip
            ("body_decay", 250.0),
            ("body_volume", 1.0),
            ("body_tone", 7000.0),
            ("body_resonance", 0.05),
            ("body_feedback", 0.2),
            ("body_hold", 8.0),
            ("body_split_freq", 100.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.75),
            ("master_volume", 0.9),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 3.0),
            ("master_mid", 0.0),
            ("master_high", 1.0),
        ]),

        // ── Frenchcore: very fast, tight, extreme drive ─────────────────────
        make_preset("Frenchcore", &[
            ("click_enabled", 1.0),
            ("click_type", 2.0),        // Punch
            ("click_volume", 0.9),
            ("click_pitch", 6000.0),
            ("click_decay", 2.0),
            ("click_filter_freq", 12000.0),
            ("body_pitch_start", 2000.0),
            ("body_pitch_end", 45.0),
            ("body_pitch_decay", 35.0),
            ("body_pitch_curve", 0.0),   // Exponential
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.8),
            ("body_distortion_type", 1.0), // HardClip
            ("body_decay", 180.0),
            ("body_volume", 1.0),
            ("body_tone", 8000.0),
            ("body_resonance", 0.1),
            ("body_feedback", 0.25),
            ("body_hold", 5.0),
            ("body_split_freq", 90.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.7),
            ("master_volume", 0.9),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 2.0),
            ("master_mid", 1.0),
            ("master_high", 2.0),
        ]),

        // ── Industrial: metallic, bitcrush, aggressive ──────────────────────
        make_preset("Industrial", &[
            ("click_enabled", 1.0),
            ("click_type", 0.0),        // Noise
            ("click_volume", 0.8),
            ("click_pitch", 3000.0),
            ("click_decay", 8.0),
            ("click_filter_freq", 5000.0),
            ("body_pitch_start", 600.0),
            ("body_pitch_end", 42.0),
            ("body_pitch_decay", 130.0),
            ("body_pitch_curve", 2.0),   // Linear
            ("body_waveform", 2.0),      // Saw
            ("body_drive", 0.6),
            ("body_distortion_type", 4.0), // Bitcrush
            ("body_decay", 400.0),
            ("body_volume", 0.9),
            ("body_tone", 4000.0),
            ("body_resonance", 0.35),
            ("body_feedback", 0.4),      // Heavy FM for metallic character
            ("body_hold", 10.0),
            ("body_split_freq", 100.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.7),
            ("master_volume", 0.8),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 1.0),
            ("master_mid", 2.0),
            ("master_high", -2.0),
        ]),

        // ── Deep Kick: slow sweep, clean, heavy low-end ────────────────────
        make_preset("Deep Kick", &[
            ("click_enabled", 1.0),
            ("click_type", 1.0),        // Sine
            ("click_volume", 0.45),
            ("click_pitch", 2000.0),
            ("click_decay", 10.0),
            ("click_filter_freq", 4000.0),
            ("body_pitch_start", 300.0),
            ("body_pitch_end", 45.0),
            ("body_pitch_decay", 250.0),
            ("body_pitch_curve", 3.0),   // S-Curve
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.1),
            ("body_distortion_type", 0.0), // Tanh
            ("body_decay", 700.0),
            ("body_volume", 0.85),
            ("body_tone", 3000.0),
            ("body_resonance", 0.0),
            ("body_feedback", 0.05),     // Very subtle FM
            ("body_hold", 20.0),         // Long hold for weight
            ("body_split_freq", 150.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 1.0),         // Full sub
            ("master_volume", 0.85),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", 5.0),
            ("master_mid", -3.0),
            ("master_high", -2.0),
        ]),

        // ── Minimal Techno: short, punchy, tight ────────────────────────────
        make_preset("Minimal Techno", &[
            ("click_enabled", 1.0),
            ("click_type", 1.0),        // Sine
            ("click_volume", 0.55),
            ("click_pitch", 3500.0),
            ("click_decay", 4.0),
            ("click_filter_freq", 6000.0),
            ("body_pitch_start", 350.0),
            ("body_pitch_end", 55.0),
            ("body_pitch_decay", 70.0),
            ("body_pitch_curve", 0.0),   // Exponential
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.08),
            ("body_distortion_type", 0.0), // Tanh
            ("body_decay", 200.0),
            ("body_volume", 0.9),
            ("body_tone", 5000.0),
            ("body_resonance", 0.0),
            ("body_feedback", 0.0),
            ("body_hold", 6.0),
            ("body_split_freq", 120.0),
            ("sub_enabled", 1.0),
            ("sub_volume", 0.8),
            ("master_volume", 0.85),
            ("master_tuning", 0.0),
            ("master_limiter", 0.0),
            ("master_low", 1.0),
            ("master_mid", 0.0),
            ("master_high", 0.0),
        ]),

        // ── Donk: bright, bouncy, fast pitch sweep ─────────────────────────
        make_preset("Donk", &[
            ("click_enabled", 1.0),
            ("click_type", 2.0),        // Punch
            ("click_volume", 0.9),
            ("click_pitch", 8000.0),
            ("click_decay", 3.0),
            ("click_filter_freq", 14000.0),
            ("body_pitch_start", 2500.0),
            ("body_pitch_end", 60.0),
            ("body_pitch_decay", 40.0),
            ("body_pitch_curve", 0.0),   // Exponential
            ("body_waveform", 0.0),      // Sine
            ("body_drive", 0.35),
            ("body_distortion_type", 0.0), // Tanh
            ("body_decay", 180.0),
            ("body_volume", 1.0),
            ("body_tone", 10000.0),
            ("body_resonance", 0.15),
            ("body_feedback", 0.1),
            ("body_hold", 4.0),
            ("body_split_freq", 100.0),
            ("sub_enabled", 0.0),
            ("sub_volume", 0.4),
            ("master_volume", 0.9),
            ("master_tuning", 0.0),
            ("master_limiter", 1.0),
            ("master_low", -2.0),
            ("master_mid", 2.0),
            ("master_high", 3.0),
        ]),
    ]
}
