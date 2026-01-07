use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: String,
    pub audio_output_device: Option<String>,
    pub sources: Vec<SourceConfig>,
    #[serde(default)]
    pub audio_engine: AudioBackend,
    pub lastfm_session: Option<LastFmSessionConfig>,
    pub discord_rpc: Option<DiscordRpcConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            audio_output_device: None,
            sources: Vec::new(),
            audio_engine: AudioBackend::default(),
            lastfm_session: None,
            discord_rpc: Some(DiscordRpcConfig::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRpcConfig {
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub show_details: bool,
    #[serde(default = "default_true")]
    pub show_state: bool,
    #[serde(default = "default_true")]
    pub show_time: bool,

    #[serde(default = "default_details_format")]
    pub details_format: String,
    #[serde(default = "default_state_format")]
    pub state_format: String,
    #[serde(default = "default_true")]
    pub activity_on_pause: bool,
    #[serde(default = "default_true")]
    pub show_artist_icon: bool,
}

impl Default for DiscordRpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_details: true,
            show_state: true,
            show_time: true,
            details_format: default_details_format(),
            state_format: default_state_format(),
            activity_on_pause: true,
            show_artist_icon: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_details_format() -> String {
    "{track}".to_string()
}

fn default_state_format() -> String {
    "{artist}".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LastFmSessionConfig {
    pub username: String,
    pub session_key: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum SourceConfig {
    Local {
        id: String,
        name: String,
        path: String,
        enabled: bool,
    },
    Subsonic {
        id: String,
        name: String,
        url: String,
        username: String,
        token: String,
        salt: String,
        enabled: bool,
    },
    Tidal {
        id: String,
        name: String,
        token: String,
        refresh_token: String,
        token_expiry: DateTime<Utc>,
        enabled: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "options")]
pub enum AudioBackend {
    #[serde(rename = "mpv")]
    Mpv(MpvConfig),
}

impl Default for AudioBackend {
    fn default() -> Self {
        Self::Mpv(MpvConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct MpvConfig {
    pub cache_mb: Option<u32>,
    pub hardware_decoding: bool,
    pub audio_device: Option<String>,
}
