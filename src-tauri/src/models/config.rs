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
    pub lastfm: Option<LastFmConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            audio_output_device: None,
            sources: Vec::new(),
            audio_engine: AudioBackend::default(),
            lastfm: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LastFmConfig {
    pub api_key: String,
    pub api_secret: String,
    pub session_key: Option<String>,
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
