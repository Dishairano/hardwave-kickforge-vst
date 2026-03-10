//! Parameter state packet sent from the KickForge VST plugin to the WebView UI.
//!
//! This mirrors the TypeScript `KickForgeState` interface exactly so the JSON
//! can be consumed directly by `window.__onKickForgePacket(data)`.

use serde::{Deserialize, Serialize};

/// Full parameter state, serialised as JSON and pushed to the WebView.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KickForgePacket {
    pub click_enabled: bool,
    pub click_volume: f32,
    pub click_pitch: f32,
    pub click_decay: f32,

    pub body_pitch_start: f32,
    pub body_pitch_end: f32,
    pub body_pitch_decay: f32,
    pub body_drive: f32,
    pub body_volume: f32,

    pub sub_enabled: bool,
    pub sub_frequency: f32,
    pub sub_volume: f32,
    pub sub_decay: f32,

    pub master_volume: f32,
    pub master_tuning: f32,
}
