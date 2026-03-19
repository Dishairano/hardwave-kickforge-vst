//! Parameter state packet sent from the KickForge VST plugin to the WebView UI.
//!
//! This mirrors the TypeScript `KickForgeState` interface exactly so the JSON
//! can be consumed directly by `window.__onKickForgePacket(data)`.

use serde::{Deserialize, Serialize};

/// State of a single modular FX slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FxSlotState {
    pub slot_type: i32,
    pub enabled: bool,
    pub p1: f32,
    pub p2: f32,
    pub p3: f32,
    pub p4: f32,
    pub p5: f32,
    pub p6: f32,
}

/// Full parameter state, serialised as JSON and pushed to the WebView.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KickForgePacket {
    // Click layer
    pub click_enabled: bool,
    pub click_type: i32,
    pub click_volume: f32,
    pub click_pitch: f32,
    pub click_decay: f32,
    pub click_filter_freq: f32,

    // Body layer
    pub body_pitch_start: f32,
    pub body_pitch_end: f32,
    pub body_pitch_decay: f32,
    pub body_pitch_curve: i32,
    pub body_waveform: i32,
    pub body_drive: f32,
    pub body_distortion_type: i32,
    pub body_decay: f32,
    pub body_volume: f32,
    pub body_tone: f32,
    pub body_resonance: f32,
    pub body_feedback: f32,
    pub body_hold: f32,
    pub body_split_freq: f32,

    // Sub layer
    pub sub_enabled: bool,
    pub sub_frequency: f32,
    pub sub_volume: f32,
    pub sub_decay: f32,

    // Noise layer
    pub noise_enabled: bool,
    pub noise_type: i32,
    pub noise_volume: f32,
    pub noise_decay: f32,
    pub noise_filter_freq: f32,

    // Layer solo
    pub click_solo: bool,
    pub body_solo: bool,
    pub sub_solo: bool,
    pub noise_solo: bool,

    // Velocity mapping
    pub vel_to_decay: f32,
    pub vel_to_pitch: f32,
    pub vel_to_drive: f32,
    pub vel_to_click: f32,

    // Modular FX rack (8 slots)
    pub fx_slots: Vec<FxSlotState>,

    // Metering (plugin -> UI only)
    pub comp_gain_reduction: f32,
    pub waveform_buffer: Vec<f32>,

    // Master
    pub master_volume: f32,
    pub master_tuning: f32,
    pub master_octave: i32,
    pub master_limiter: bool,
    pub master_low: f32,
    pub master_mid: f32,
    pub master_high: f32,
}
