use crate::models::{
    entities::{PlayerEvent, Playlist, UnifiedSearchResult},
    Album, Artist, PlayerState, Track,
};
use async_trait::async_trait;
use tokio::sync::broadcast;

pub enum AudioStream {
    Url(String),
    #[allow(dead_code)]
    Bytes(Vec<u8>),
}

#[async_trait]
pub trait LibraryProvider: Send + Sync {
    fn id(&self) -> &str;
    #[allow(dead_code)]
    fn name(&self) -> &str;

    async fn get_recent_albums(&self, limit: u32) -> Result<Vec<Album>, String>;
    async fn get_favorites(&self) -> Result<Vec<Track>, String>;

    async fn search(&self, query: &str) -> Result<UnifiedSearchResult, String>;

    async fn get_artist(&self, id: &str) -> Result<Artist, String>;
    async fn get_artist_albums(&self, artist_id: &str) -> Result<Vec<Album>, String>;
    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Track>, String>;
    async fn get_track(&self, track_id: &str) -> Result<Track, String>;
    async fn set_track_liked(&self, _track_id: &str, _liked: bool) -> Result<(), String> {
        Err("Not supported".to_string())
    }

    async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        Ok(vec![])
    }
    async fn create_playlist(&self, _name: &str) -> Result<Playlist, String> {
        Err("Not supported".to_string())
    }
    async fn delete_playlist(&self, _id: &str) -> Result<(), String> {
        Err("Not supported".to_string())
    }
    async fn get_playlist_tracks(&self, _id: &str) -> Result<Vec<Track>, String> {
        Ok(vec![])
    }
    async fn add_to_playlist(&self, _playlist_id: &str, _track_id: &str) -> Result<(), String> {
        Err("Not supported".to_string())
    }
    async fn remove_from_playlist(
        &self,
        _playlist_id: &str,
        _track_id: &str,
    ) -> Result<(), String> {
        Err("Not supported".to_string())
    }

    async fn resolve_stream(&self, track_id: &str) -> Result<AudioStream, String>;

    async fn scan(&self) -> Result<(), String> {
        Ok(())
    }

    async fn add_root(&self, _path: &str) -> Result<(), String> {
        Err("Not supported".to_string())
    }
}

#[async_trait]
pub trait AudioEngine: Send + Sync {
    async fn load(&self, stream: AudioStream, auto_play: bool) -> Result<(), String>;
    async fn play(&self) -> Result<(), String>;
    async fn pause(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn seek(&self, seconds: f64) -> Result<(), String>;
    async fn set_volume(&self, vol: f32) -> Result<(), String>;

    async fn get_state(&self) -> PlayerState;

    fn subscribe(&self) -> broadcast::Receiver<PlayerEvent>;
}
