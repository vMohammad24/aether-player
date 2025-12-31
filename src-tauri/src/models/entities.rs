use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, Type, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    #[sqlx(default)]
    pub provider_id: Option<String>,
    pub title: String,

    pub artist_id: String,
    #[sqlx(default)]
    pub artist_name: String,
    pub album_id: String,
    #[sqlx(default)]
    pub album_title: String,

    pub duration_sec: u32,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u16>,

    pub genre: Option<String>,
    pub bitrate: Option<u32>,
    pub play_count: u32,
    pub liked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist_id: String,
    #[sqlx(default)]
    pub artist_name: String,
    pub cover_art: Option<String>,
    pub year: Option<u16>,
    #[sqlx(default)]
    pub track_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub owner: String,
    #[sqlx(default)]
    pub track_count: u32,
    pub cover_art: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedSearchResult {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(tag = "type", content = "data")]
pub enum PlayerEvent {
    TimeUpdate(f64),
    DurationChange(f64),
    Paused,
    Playing,
    Ended,
    Error(String),
}
