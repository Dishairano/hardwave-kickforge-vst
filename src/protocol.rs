//! Parameter state packet sent from the KickForge VST plugin to the WebView UI.
//!
//! This mirrors the TypeScript `KickForgeState` interface exactly so the JSON
//! can be consumed directly by `window.__onKickForgePacket(data)`.

use serde::{Deserialize, Serialize};

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

    // Sub layer
    pub sub_enabled: bool,
    pub sub_frequency: f32,
    pub sub_volume: f32,
    pub sub_decay: f32,

    // Master
    pub master_volume: f32,
    pub master_tuning: f32,
    pub master_octave: i32,
    pub master_limiter: bool,
    pub master_low: f32,
    pub master_mid: f32,
    pub master_high: f32,
}
