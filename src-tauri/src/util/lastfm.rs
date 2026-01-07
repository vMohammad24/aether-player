use crate::models::entities::PlayerEvent;
use crate::queue::QueueManager;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const API_ROOT: &str = "https://ws.audioscrobbler.com/2.0/";
const API_KEY: &str = env!("LASTFM_API_KEY");
const API_SECRET: &str = env!("LASTFM_API_SECRET");

pub fn start_scrobbling_service(
    queue: Arc<QueueManager>,
    lastfm: Arc<Mutex<Option<LastFmClient>>>,
) {
    tauri::async_runtime::spawn(async move {
        let mut rx = queue.player.subscribe();
        let mut current_track_id: Option<String> = None;
        let mut scrobbled = false;

        while let Ok(event) = rx.recv().await {
            let client = {
                let guard = lastfm.lock().await;
                guard.clone()
            };

            if let Some(client) = client {
                match &event {
                    PlayerEvent::Playing => {
                        if let Some(track) = queue.current_track().await {
                            if current_track_id.as_deref() != Some(&track.id) {
                                current_track_id = Some(track.id.clone());
                                scrobbled = false;

                                let client = client.clone();
                                let artist = track.artist_name.clone();
                                let title = track.title.clone();
                                let album = track.album_title.clone();

                                tauri::async_runtime::spawn(async move {
                                    if let Err(e) = client
                                        .update_now_playing(&artist, &title, Some(&album))
                                        .await
                                    {
                                        log::warn!("Last.fm Now Playing error: {}", e);
                                    }
                                });
                            }
                        }
                    }
                    PlayerEvent::TimeUpdate(pos) => {
                        if !scrobbled {
                            if let Some(track) = queue.current_track().await {
                                if current_track_id.as_deref() == Some(&track.id) {
                                    let duration = track.duration_sec as f64;
                                    if duration > 30.0 {
                                        let threshold = (duration / 2.0).min(240.0);
                                        if *pos >= threshold {
                                            scrobbled = true;
                                            let client = client.clone();
                                            let artist = track.artist_name.clone();
                                            let title = track.title.clone();
                                            let album = track.album_title.clone();
                                            let timestamp = chrono::Utc::now().timestamp();

                                            tauri::async_runtime::spawn(async move {
                                                if let Err(e) = client
                                                    .scrobble(
                                                        &artist,
                                                        &title,
                                                        timestamp,
                                                        Some(&album),
                                                    )
                                                    .await
                                                {
                                                    log::error!("Last.fm Scrobble error: {}", e);
                                                } else {
                                                    log::info!("Scrobbled: {} - {}", artist, title);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

#[derive(Clone)]
pub struct LastFmClient {
    username: Option<String>,
    session_key: Option<String>,
    client: Client,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    #[serde(rename = "#text")]
    pub url: String,
    pub size: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tag {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tags {
    pub tag: Vec<Tag>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Stats {
    pub listeners: String,
    pub playcount: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bio {
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimilarArtist {
    pub name: String,
    pub url: String,
    pub image: Option<Vec<Image>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Similar {
    pub artist: Vec<SimilarArtist>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArtistInfo {
    pub name: String,
    pub mbid: Option<String>,
    pub url: String,
    pub image: Option<Vec<Image>>,
    pub stats: Option<Stats>,
    pub similar: Option<Similar>,
    pub tags: Option<Tags>,
    pub bio: Option<Bio>,
}

#[derive(Deserialize)]
struct ArtistInfoResponse {
    artist: ArtistInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrackArtist {
    pub name: String,
    pub mbid: Option<String>,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrackAlbum {
    pub artist: String,
    pub title: String,
    pub mbid: Option<String>,
    pub url: String,
    pub image: Option<Vec<Image>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TrackInfo {
    pub name: String,
    pub mbid: Option<String>,
    pub url: String,
    pub duration: Option<String>,
    pub listeners: String,
    pub playcount: String,
    pub artist: TrackArtist,
    pub album: Option<TrackAlbum>,
    pub toptags: Option<Tags>,
    pub wiki: Option<Bio>,
    pub userplaycount: Option<String>,
    pub userloved: Option<String>,
}

#[derive(Deserialize)]
struct TrackInfoResponse {
    track: TrackInfo,
}

#[derive(Deserialize)]
pub struct LastFmSession {
    pub name: String,
    pub key: String,
}

#[derive(Deserialize)]
struct SessionResponse {
    session: LastFmSession,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

impl LastFmClient {
    pub fn new(username: Option<String>, session_key: Option<String>) -> Self {
        Self {
            username,
            session_key,
            client: Client::new(),
        }
    }

    pub fn set_session_key(&mut self, session_key: String) {
        self.session_key = Some(session_key);
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    fn sign_params(&self, params: &mut HashMap<String, String>) {
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();

        let mut sig_base = String::new();
        for key in keys {
            if key == "format" || key == "callback" {
                continue;
            }
            sig_base.push_str(key);
            sig_base.push_str(&params[key]);
        }
        sig_base.push_str(API_SECRET);

        let digest = md5::compute(sig_base);
        params.insert("api_sig".to_string(), format!("{:x}", digest));
    }

    pub async fn get_token(&self) -> Result<String> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "auth.getToken".to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());

        self.sign_params(&mut params);
        params.insert("format".to_string(), "json".to_string());

        let res = self
            .client
            .get(API_ROOT)
            .query(&params)
            .send()
            .await
            .context("Failed to send Last.fm getToken request")?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("Last.fm API Error: {}", res.status()));
        }

        let data: TokenResponse = res
            .json()
            .await
            .context("Failed to parse Last.fm token response")?;
        Ok(data.token)
    }

    pub async fn get_session(&self, token: &str) -> Result<LastFmSession> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "auth.getSession".to_string());
        params.insert("token".to_string(), token.trim().to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());

        self.sign_params(&mut params);
        params.insert("format".to_string(), "json".to_string());

        let res = self
            .client
            .get(API_ROOT)
            .query(&params)
            .send()
            .await
            .context("Failed to send Last.fm getSession request")?;

        let status = res.status();

        if !status.is_success() {
            let url = res.url().to_string();
            let text = res.text().await.unwrap_or_default();

            return Err(anyhow::anyhow!(
                "Last.fm API Error: {} - {}, URL: {}",
                status,
                text,
                url
            ));
        }

        let data: SessionResponse = res
            .json()
            .await
            .context("Failed to parse Last.fm session response")?;

        Ok(data.session)
    }

    pub async fn get_artist_info(&self, artist: &str) -> Result<ArtistInfo> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "artist.getInfo".to_string());
        params.insert("artist".to_string(), artist.to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());
        params.insert("format".to_string(), "json".to_string());
        params.insert("autocorrect".to_string(), "1".to_string());

        if let Some(username) = &self.username {
            params.insert("username".to_string(), username.clone());
        }

        let res = self
            .client
            .get(API_ROOT)
            .query(&params)
            .send()
            .await
            .context("Failed to send Last.fm request")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Last.fm API Error {}: {}", status, text));
        }

        let data: ArtistInfoResponse = res
            .json()
            .await
            .context("Failed to parse Last.fm response")?;
        Ok(data.artist)
    }

    pub async fn get_track_info(&self, artist: &str, track: &str) -> Result<TrackInfo> {
        let mut params = HashMap::new();
        params.insert("method".to_string(), "track.getInfo".to_string());
        params.insert("artist".to_string(), artist.to_string());
        params.insert("track".to_string(), track.to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());
        params.insert("format".to_string(), "json".to_string());
        params.insert("autocorrect".to_string(), "1".to_string());

        if let Some(username) = &self.username {
            params.insert("username".to_string(), username.clone());
        }

        let res = self
            .client
            .get(API_ROOT)
            .query(&params)
            .send()
            .await
            .context("Failed to send Last.fm request")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Last.fm API Error {}: {}", status, text));
        }

        let data: TrackInfoResponse = res
            .json()
            .await
            .context("Failed to parse Last.fm response")?;
        Ok(data.track)
    }

    pub async fn scrobble(
        &self,
        artist: &str,
        track: &str,
        timestamp: i64,
        album: Option<&str>,
    ) -> Result<()> {
        let sk = self
            .session_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Last.fm session key not set"))?;

        let mut params = HashMap::new();
        params.insert("method".to_string(), "track.scrobble".to_string());
        params.insert("artist".to_string(), artist.to_string());
        params.insert("track".to_string(), track.to_string());
        params.insert("timestamp".to_string(), timestamp.to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());
        params.insert("sk".to_string(), sk.clone());

        if let Some(a) = album {
            params.insert("album".to_string(), a.to_string());
        }

        self.sign_params(&mut params);
        params.insert("format".to_string(), "json".to_string());

        let res = self
            .client
            .post(API_ROOT)
            .form(&params)
            .send()
            .await
            .context("Failed to send Last.fm scrobble request")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Last.fm Scrobble Error {}: {}",
                status,
                text
            ));
        }

        Ok(())
    }

    pub async fn update_now_playing(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
    ) -> Result<()> {
        let sk = self
            .session_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Last.fm session key not set"))?;

        let mut params = HashMap::new();
        params.insert("method".to_string(), "track.updateNowPlaying".to_string());
        params.insert("artist".to_string(), artist.to_string());
        params.insert("track".to_string(), track.to_string());
        params.insert("api_key".to_string(), API_KEY.to_string());
        params.insert("sk".to_string(), sk.clone());

        if let Some(a) = album {
            params.insert("album".to_string(), a.to_string());
        }

        self.sign_params(&mut params);
        params.insert("format".to_string(), "json".to_string());

        let res = self
            .client
            .post(API_ROOT)
            .form(&params)
            .send()
            .await
            .context("Failed to send Last.fm now playing request")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Last.fm Now Playing Error {}: {}",
                status,
                text
            ));
        }

        Ok(())
    }
}
