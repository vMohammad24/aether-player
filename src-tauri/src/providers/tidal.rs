use crate::models::entities::{
    Album, Artist, Genre, LibraryStats, Playlist, Track, UnifiedSearchResult,
};
use crate::traits::{AudioStream, LibraryProvider};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use log::info;
use moka::future::Cache;
use rand::seq::SliceRandom;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

const TIDAL_AUTH_BASE_URI: &str = "https://auth.tidal.com/v1";
const TIDAL_API_URL: &str = "https://api.tidal.com/v1";
const TIDAL_API_V2_URL: &str = "https://api.tidal.com/v2";
const TIDAL_OPENAPI_V2_URL: &str = "https://openapi.tidal.com/v2";
const TIDAL_DESKTOP_V1_URL: &str = "https://desktop.tidal.com/v1";
const TIDAL_DESKTOP_V2_URL: &str = "https://desktop.tidal.com/v2";
const TIDAL_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) TIDAL/1.8.0-beta Chrome/126.0.6478.127 Electron/31.2.1 Safari/537.36";
const TIDAL_CLIENT_ID: &str = env!("TIDAL_CLIENT_ID");
const TIDAL_CLIENT_SECRET: &str = env!("TIDAL_CLIENT_SECRET");

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct DeviceAuthPending {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_at: DateTime<Utc>,
    pub interval: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceAuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: i64,
    interval: u32,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    scope: String,
}

#[derive(Deserialize)]
struct UserProfile {
    #[serde(rename = "userId")]
    user_id: u64,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum ApiVersion {
    V1,
    V2,
    OpenApi,
    Desktop,
    DesktopV2,
}

impl ApiVersion {
    fn base_url(&self) -> &'static str {
        match self {
            ApiVersion::V1 => TIDAL_API_URL,
            ApiVersion::V2 => TIDAL_API_V2_URL,
            ApiVersion::OpenApi => TIDAL_OPENAPI_V2_URL,
            ApiVersion::Desktop => TIDAL_DESKTOP_V1_URL,
            ApiVersion::DesktopV2 => TIDAL_DESKTOP_V2_URL,
        }
    }
}

#[derive(Clone, Debug)]
enum CachedItem {
    AlbumList(Vec<Album>),
    TrackList(Vec<Track>),
    PlaylistList(Vec<Playlist>),
    SingleArtist(Artist),
    SingleAlbum(Album),
    SingleTrack(Track),
    SearchResult(UnifiedSearchResult),
}

#[derive(Clone)]
pub struct TidalProvider {
    id: String,
    name: String,
    credentials: Arc<RwLock<TidalCredentials>>,
    client: Client,

    cache: Cache<String, CachedItem>,

    favorite_ids: Arc<RwLock<HashSet<String>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TidalCredentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub country_code: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TidalPage<T> {
    items: Vec<T>,
    limit: usize,
    offset: usize,
    #[serde(alias = "totalNumberOfItems")]
    total_items: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct TidalArtist {
    id: u64,
    name: String,
    picture: Option<String>,
    #[serde(alias = "selectedAlbumCoverFallback")]
    picture_fallback: Option<String>,
    bio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TidalAlbum {
    id: u64,
    title: String,
    artist: Option<TidalArtist>,
    artists: Option<Vec<TidalArtist>>,
    cover: Option<String>,
    #[serde(alias = "releaseDate")]
    release_date: Option<String>,
    #[serde(alias = "numberOfTracks")]
    number_of_tracks: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TidalTrack {
    id: u64,
    title: String,
    duration: u32,
    #[serde(alias = "trackNumber")]
    track_number: u32,
    #[serde(alias = "volumeNumber")]
    disc_number: u32,
    artist: Option<TidalArtist>,
    album: Option<TidalAlbum>,
    #[serde(alias = "streamStartDate")]
    stream_start_date: Option<String>,
    popularity: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TidalPlaylist {
    uuid: String,
    title: String,
    description: Option<String>,
    image: Option<String>,
    #[serde(alias = "numberOfTracks")]
    number_of_tracks: u32,
    created: Option<String>,
    creator: Option<TidalCreator>,
}

#[derive(Debug, Deserialize)]
struct TidalCreator {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TidalFavoriteItem {
    created: String,
    item: TidalTrack,
}

#[derive(Debug, Deserialize)]
struct TidalPlaybackInfo {
    #[serde(alias = "trackId")]
    track_id: u64,
    #[serde(alias = "assetPresentation")]
    asset_presentation: String,
    #[serde(alias = "audioMode")]
    audio_mode: String,
    #[serde(alias = "audioQuality")]
    audio_quality: String,
    #[serde(alias = "manifestMimeType")]
    manifest_mime_type: String,
    #[serde(alias = "manifestHash")]
    manifest_hash: String,
}

#[derive(Debug, Deserialize)]
struct TidalSearchResults {
    artists: Option<TidalPage<TidalArtist>>,
    albums: Option<TidalPage<TidalAlbum>>,
    tracks: Option<TidalPage<TidalTrack>>,
}

#[derive(Debug, Deserialize)]
struct TidalHomePageResponse {
    title: String,
    #[serde(alias = "itemLayout")]
    item_layout: Option<String>,
    items: Vec<TidalHomePageItem>,
}

#[derive(Debug, Deserialize)]
struct TidalHomePageItem {
    #[serde(rename = "type")]
    item_type: String,
    data: TidalAlbum,
}

fn get_image_url(id: &str, width: u32, height: u32) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    let path = id.replace("-", "/");
    Some(format!(
        "https://resources.tidal.com/images/{}/{}x{}.jpg",
        path, width, height
    ))
}

impl TidalProvider {
    pub async fn new(id: String, name: String, credentials: TidalCredentials) -> Result<Self> {
        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(std::time::Duration::from_secs(3600))
            .build();

        let provider = Self {
            id,
            name,
            credentials: Arc::new(RwLock::new(credentials)),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent(TIDAL_USER_AGENT)
                .build()
                .context("Failed to build HTTP client")?,
            cache,
            favorite_ids: Arc::new(RwLock::new(HashSet::new())),
        };
        let prov = provider.clone();
        tokio::spawn(async move {
            info!("Loading favorite track IDs...");
            match provider.get_favorite_ids().await {
                Ok(ids) => {
                    let mut fav_ids = provider.favorite_ids.write().await;
                    fav_ids.extend(ids);
                    info!("Loaded {} favorite tracks", fav_ids.len());
                }
                Err(e) => {
                    log::warn!("Failed to load favorite IDs during initialization: {}", e);
                }
            }
        });

        Ok(prov)
    }

    pub async fn get_favorite_ids(&self) -> Result<Vec<String>, String> {
        let user_id = {
            let creds = self.credentials.read().await;
            creds.user_id.clone().ok_or("No user ID")?
        };

        let path = format!("users/{}/favorites/ids", user_id);
        let res: serde_json::Value = self
            .request(reqwest::Method::GET, &path, None, None, ApiVersion::Desktop)
            .await
            .map_err(|e| e.to_string())?;

        let ids = res
            .get("TRACK")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|id| id.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ids)
    }

    fn map_track(t: &TidalTrack, favorites: &HashSet<String>) -> Track {
        let artist_name = t
            .artist
            .as_ref()
            .map(|a| a.name.clone())
            .unwrap_or_default();
        let artist_id = t
            .artist
            .as_ref()
            .map(|a| a.id.to_string())
            .unwrap_or_default();
        let album_title = t
            .album
            .as_ref()
            .map(|a| a.title.clone())
            .unwrap_or_default();
        let album_id = t
            .album
            .as_ref()
            .map(|a| a.id.to_string())
            .unwrap_or_default();

        let year = t
            .stream_start_date
            .as_ref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse::<u16>().ok());

        let track_id = t.id.to_string();
        let liked = favorites.contains(&track_id);

        Track {
            id: track_id,
            provider_id: Some("tidal".to_string()),
            title: t.title.clone(),
            artist_id,
            artist_name,
            album_id,
            album_title,
            duration_sec: t.duration,
            track_number: Some(t.track_number),
            disc_number: Some(t.disc_number),
            year,
            genre: None,
            bitrate: None,
            play_count: 0,
            liked,
        }
    }

    fn map_album(a: &TidalAlbum) -> Album {
        let year = a
            .release_date
            .as_ref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse::<u16>().ok());
        let artist = a
            .artist
            .as_ref()
            .or_else(|| a.artists.as_ref().and_then(|artists| artists.first()));
        Album {
            id: a.id.to_string(),
            title: a.title.clone(),
            artist_id: artist
                .as_ref()
                .map(|ar| ar.id.to_string())
                .unwrap_or_default(),
            artist_name: artist
                .as_ref()
                .map(|ar| ar.name.clone())
                .unwrap_or_default(),
            cover_art: a.cover.as_ref().and_then(|c| get_image_url(c, 640, 640)),
            year,
            track_count: a.number_of_tracks,
        }
    }

    fn map_artist(a: &TidalArtist) -> Artist {
        let picture = a.picture.as_ref().or_else(|| a.picture_fallback.as_ref());
        Artist {
            id: a.id.to_string(),
            name: a.name.clone(),
            bio: a.bio.clone(),
            image_url: picture.and_then(|p| get_image_url(p, 750, 750)),
        }
    }

    fn map_playlist(p: &TidalPlaylist) -> Playlist {
        let created_at = p
            .created
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Playlist {
            id: p.uuid.clone(),
            name: p.title.clone(),
            owner: p
                .creator
                .as_ref()
                .and_then(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            track_count: p.number_of_tracks,
            cover_art: p.image.as_ref().and_then(|i| get_image_url(i, 640, 640)),
            created_at,
        }
    }

    pub async fn ensure_valid_token(&self) -> Result<()> {
        let mut creds = self.credentials.write().await;
        let now = Utc::now();

        if let Some(expires) = creds.expires_at {
            if expires > now + Duration::seconds(60) {
                return Ok(());
            }
        }

        if let Some(refresh_token) = &creds.refresh_token {
            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("client_id", TIDAL_CLIENT_ID);
            params.insert("client_secret", TIDAL_CLIENT_SECRET);
            params.insert("refresh_token", refresh_token.as_str());
            params.insert("grant_type", "refresh_token");
            params.insert("scope", "r_usr w_usr w_sub");

            let res = self
                .client
                .post(format!("{}/oauth2/token", TIDAL_AUTH_BASE_URI))
                .form(&params)
                .send()
                .await?;

            if !res.status().is_success() {
                return Err(anyhow!("Failed to refresh token: {}", res.status()));
            }

            let data: TokenResponse = res.json().await?;
            creds.access_token = Some(data.access_token);
            if let Some(rt) = data.refresh_token {
                creds.refresh_token = Some(rt);
            }
            creds.expires_at = Some(Utc::now() + Duration::seconds(data.expires_in));
        } else {
            return Err(anyhow!("Session expired and no refresh token available"));
        }

        Ok(())
    }

    pub async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<HashMap<String, String>>,
        data: Option<serde_json::Value>,
        api_version: ApiVersion,
    ) -> Result<T> {
        let creds = self.credentials.read().await;
        let use_oauth = creds.access_token.is_some();
        if !use_oauth {
            return Err(anyhow!("Session is not valid. Please login first."));
        }
        drop(creds);

        self.ensure_valid_token()
            .await
            .context("Failed to ensure valid OAuth token")?;

        let creds = self.credentials.read().await;
        let access_token = creds
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow!("OAuth token missing"))?;
        let country_code = &creds.country_code;

        let base_url = api_version.base_url();
        let url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let mut query_params = params.unwrap_or_default();
        query_params.insert("countryCode".to_string(), country_code.clone());
        query_params.insert("deviceType".to_string(), "DESKTOP".to_string());

        let mut request = self
            .client
            .request(method, &url)
            .query(&query_params)
            .header("X-Tidal-Client-Version", "2026.1.5")
            .header("Authorization", format!("Bearer {}", access_token));

        request = match api_version {
            ApiVersion::V2 => request.header("Accept", "application/vnd.tidal.v1+json"),
            _ => request.header("Accept", "application/json"),
        };

        if let Some(body) = data {
            request = request.json(&body);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("TIDAL API request to {}: {}", url, error_text);
            return Err(anyhow!("API Error : {}", error_text));
        }
        let text = response.text().await?;
        // info!("TIDAL API request to {} succeeded, Data: {:?}", url, &text);
        serde_json::from_str::<T>(&text).context("Failed to parse response")
    }

    pub async fn start_device_auth() -> Result<DeviceAuthPending> {
        let client = Client::new();
        let mut params = HashMap::new();
        params.insert("client_id", TIDAL_CLIENT_ID);
        params.insert("client_secret", TIDAL_CLIENT_SECRET);
        params.insert("scope", "r_usr w_usr w_sub");

        let res = client
            .post(format!(
                "{}/oauth2/device_authorization",
                TIDAL_AUTH_BASE_URI
            ))
            .form(&params)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Device auth init failed: {}", res.text().await?));
        }
        let data: DeviceAuthResponse = res.json().await?;
        Ok(DeviceAuthPending {
            device_code: data.device_code,
            user_code: data.user_code,
            verification_uri: data.verification_uri,
            verification_uri_complete: data.verification_uri_complete,
            expires_at: Utc::now() + Duration::seconds(data.expires_in),
            interval: data.interval,
        })
    }

    pub async fn poll_device_token(pending: &DeviceAuthPending) -> Result<TidalCredentials> {
        let client = Client::new();
        loop {
            if Utc::now() > pending.expires_at {
                return Err(anyhow!("Device code expired"));
            }

            let mut params = HashMap::new();
            params.insert("client_id", TIDAL_CLIENT_ID);
            params.insert("client_secret", TIDAL_CLIENT_SECRET);
            params.insert("device_code", pending.device_code.as_str());
            params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
            params.insert("scope", "r_usr w_usr w_sub");

            let res = client
                .post(format!("{}/oauth2/token", TIDAL_AUTH_BASE_URI))
                .form(&params)
                .send()
                .await?;

            if res.status().is_success() {
                let token_data: TokenResponse = res.json().await?;

                let user_res = client
                    .get("https://login.tidal.com/oauth2/me")
                    .bearer_auth(&token_data.access_token)
                    .send()
                    .await?;
                let user_info: UserProfile = user_res.json().await?;

                return Ok(TidalCredentials {
                    access_token: Some(token_data.access_token),
                    refresh_token: token_data.refresh_token,
                    expires_at: Some(Utc::now() + Duration::seconds(token_data.expires_in)),
                    user_id: Some(user_info.user_id.to_string()),
                    country_code: user_info.country_code.unwrap_or_else(|| "US".into()),
                    scopes: token_data
                        .scope
                        .split_whitespace()
                        .map(String::from)
                        .collect(),
                });
            } else if res.status() == StatusCode::BAD_REQUEST {
                let err = res.text().await?;
                if err.contains("authorization_pending") {
                    tokio::time::sleep(std::time::Duration::from_secs(pending.interval as u64))
                        .await;
                    continue;
                }
                return Err(anyhow!("Auth failed: {}", err));
            }
            return Err(anyhow!("Unexpected status: {}", res.status()));
        }
    }
}

#[async_trait::async_trait]
impl LibraryProvider for TidalProvider {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    async fn scan(&self) -> Result<(), String> {
        info!("Invalidating Tidal cache...");
        self.cache.invalidate_all();

        info!("Loading favorite track IDs...");
        match self.get_favorite_ids().await {
            Ok(ids) => {
                let mut fav_ids = self.favorite_ids.write().await;
                fav_ids.clear();
                fav_ids.extend(ids);
                info!("Loaded {} favorite tracks", fav_ids.len());
            }
            Err(e) => {
                log::warn!("Failed to load favorite IDs: {}", e);
            }
        }

        Ok(())
    }

    async fn get_artist_albums(&self, artist_id: &str) -> Result<Vec<Album>, String> {
        let key = format!("artist_albums:{}", artist_id);
        if let Some(CachedItem::AlbumList(albums)) = self.cache.get(&key).await {
            return Ok(albums);
        }

        let mut params = HashMap::new();
        params.insert("limit".to_string(), "50".to_string());

        let res: TidalPage<TidalAlbum> = self
            .request(
                reqwest::Method::GET,
                &format!("artists/{}/albums", artist_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let albums: Vec<Album> = res.items.iter().map(Self::map_album).collect();
        self.cache
            .insert(key, CachedItem::AlbumList(albums.clone()))
            .await;
        Ok(albums)
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Track>, String> {
        let key = format!("album_tracks:{}", album_id);
        if let Some(CachedItem::TrackList(tracks)) = self.cache.get(&key).await {
            return Ok(tracks);
        }

        let mut params = HashMap::new();
        params.insert("limit".to_string(), "50".to_string());

        let res: TidalPage<TidalTrack> = self
            .request(
                reqwest::Method::GET,
                &format!("albums/{}/tracks", album_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let favorites = self.favorite_ids.read().await;
        let tracks: Vec<Track> = res
            .items
            .iter()
            .map(|t| Self::map_track(t, &favorites))
            .collect();
        self.cache
            .insert(key, CachedItem::TrackList(tracks.clone()))
            .await;
        Ok(tracks)
    }

    async fn get_recent_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let key = format!("recent_albums:{}", limit);
        if let Some(CachedItem::AlbumList(albums)) = self.cache.get(&key).await {
            return Ok(albums);
        }

        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("offset".to_string(), "0".to_string());
        params.insert("locale".to_string(), "en_US".to_string());
        params.insert("platform".to_string(), "DESKTOP".to_string());

        let res: TidalHomePageResponse = self
            .request(
                reqwest::Method::GET,
                "home/pages/NEW_ALBUM_SUGGESTIONS/view-all",
                Some(params),
                None,
                ApiVersion::V2,
            )
            .await
            .map_err(|e| e.to_string())?;

        let albums: Vec<Album> = res
            .items
            .iter()
            .filter(|item| item.item_type == "ALBUM")
            .map(|item| Self::map_album(&item.data))
            .collect();

        self.cache
            .insert(key, CachedItem::AlbumList(albums.clone()))
            .await;
        Ok(albums)
    }

    async fn get_random_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let key = format!("recent_albums:{}", limit);
        if let Some(CachedItem::AlbumList(mut albums)) = self.cache.get(&key).await {
            albums.shuffle(&mut rand::rng());
            return Ok(albums);
        }

        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("offset".to_string(), "0".to_string());
        params.insert("locale".to_string(), "en_US".to_string());
        params.insert("platform".to_string(), "DESKTOP".to_string());

        let res: TidalHomePageResponse = self
            .request(
                reqwest::Method::GET,
                "home/pages/BECAUSE_YOU_LISTENED_TO_ALBUM/view-all",
                Some(params),
                None,
                ApiVersion::V2,
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut albums: Vec<Album> = res
            .items
            .iter()
            .filter(|item| item.item_type == "ALBUM")
            .map(|item| Self::map_album(&item.data))
            .collect();

        self.cache
            .insert(key, CachedItem::AlbumList(albums.clone()))
            .await;
        albums.shuffle(&mut rand::rng());
        Ok(albums.into_iter().take(limit as usize).collect())
    }

    async fn get_most_played_tracks(&self, limit: u32) -> Result<Vec<Track>, String> {
        let key = format!("most_played:{}", limit);
        info!("Fetching most played tracks with key: {}", key);
        if let Some(CachedItem::TrackList(tracks)) = self.cache.get(&key).await {
            info!("Cache hit for most played tracks");
            return Ok(tracks);
        }

        let mut params = HashMap::new();
        params.insert("locale".to_string(), "en_US".to_string());
        params.insert("platform".to_string(), "DESKTOP".to_string());
        params.insert("limit".to_string(), limit.to_string());

        let res_value: serde_json::Value = self
            .request(
                reqwest::Method::GET,
                "home/pages/HISTORY_MIXES/view-all",
                Some(params),
                None,
                ApiVersion::V2,
            )
            .await
            .map_err(|e| {
                log::error!("Failed to fetch HISTORY_MIXES: {}", e);
                e.to_string()
            })?;

        info!(
            "Response keys: {:?}",
            res_value.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );

        let items = res_value
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "No items array in response".to_string())?;

        info!(
            "Fetched mixes for most played tracks, found {} items",
            items.len()
        );

        for (idx, item) in items.iter().enumerate() {
            let item_type = item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let mix_type = item
                .get("data")
                .and_then(|d| d.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            info!(
                "Mix {}: type='{}', data.type='{}'",
                idx, item_type, mix_type
            );
        }
        let mix_item = items
            .iter()
            .find(|item| {
                item.get("type").and_then(|v| v.as_str()) == Some("MIX")
                    && item
                        .get("data")
                        .and_then(|d| d.get("type"))
                        .and_then(|v| v.as_str())
                        == Some("HISTORY_ALLTIME_MIX")
            })
            .ok_or_else(|| {
                log::info!("No HISTORY_ALLTIME_MIX found in Tidal response");
                "No HISTORY_ALLTIME_MIX found".to_string()
            })?;

        let mix_id = mix_item
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "No mix ID found".to_string())?;

        info!("Fetching most played tracks from mix ID: {}", mix_id);
        let res: serde_json::Value = self
            .request(
                reqwest::Method::GET,
                &format!("pages/mix?mixId={}", mix_id),
                None,
                None,
                ApiVersion::Desktop,
            )
            .await
            .map_err(|e| e.to_string())?;

        let track_data: Vec<TidalTrack> = res
            .get("rows")
            .and_then(|rows| rows.get(1))
            .and_then(|row| row.get("modules"))
            .and_then(|modules| modules.get(0))
            .and_then(|module| module.get("pagedList"))
            .and_then(|paged_list| paged_list.get("items"))
            .and_then(|items| items.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| serde_json::from_value::<TidalTrack>(item.clone()).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let favorites = self.favorite_ids.read().await;
        let tracks: Vec<Track> = track_data
            .iter()
            .map(|t| Self::map_track(t, &favorites))
            .collect();
        self.cache
            .insert(key, CachedItem::TrackList(tracks.clone()))
            .await;
        Ok(tracks)
    }

    async fn get_library_stats(&self) -> Result<LibraryStats, String> {
        Ok(LibraryStats::default())
    }

    async fn get_genres(&self) -> Result<Vec<Genre>, String> {
        let response: Vec<serde_json::Value> = self
            .request(reqwest::Method::GET, "genres", None, None, ApiVersion::V1)
            .await
            .map_err(|e| e.to_string())?;

        let genres = response
            .iter()
            .map(|v| Genre {
                name: v["path"].as_str().unwrap_or("Unknown").to_string(),
                track_count: 0,
            })
            .collect();
        Ok(genres)
    }

    async fn get_genre_tracks(&self, genre: &str) -> Result<Vec<Track>, String> {
        let params = HashMap::from([("limit".to_string(), "50".to_string())]);
        let res: TidalPage<TidalTrack> = self
            .request(
                reqwest::Method::GET,
                &format!("genres/{}/tracks", genre.to_lowercase()),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let favorites = self.favorite_ids.read().await;
        Ok(res
            .items
            .iter()
            .map(|t| Self::map_track(t, &favorites))
            .collect())
    }

    async fn get_favorites(&self) -> Result<Vec<Track>, String> {
        let user_id = {
            let creds = self.credentials.read().await;
            creds.user_id.clone().ok_or("No user ID")?
        };

        let key = format!("favorites:{}", user_id);
        if let Some(CachedItem::TrackList(tracks)) = self.cache.get(&key).await {
            return Ok(tracks);
        }

        let mut params = HashMap::new();
        params.insert("limit".to_string(), "100".to_string());

        let res: TidalPage<TidalFavoriteItem> = self
            .request(
                reqwest::Method::GET,
                &format!("users/{}/favorites/tracks", user_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let favorites = self.favorite_ids.read().await;
        let tracks: Vec<Track> = res
            .items
            .iter()
            .map(|t| Self::map_track(&t.item, &favorites))
            .collect();

        self.cache
            .insert(key, CachedItem::TrackList(tracks.clone()))
            .await;
        Ok(tracks)
    }

    async fn search(&self, query: &str) -> Result<UnifiedSearchResult, String> {
        let key = format!("search:{}", query);
        if let Some(CachedItem::SearchResult(res)) = self.cache.get(&key).await {
            return Ok(res);
        }

        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());
        params.insert("limit".to_string(), "500".to_string());
        params.insert("types".to_string(), "ARTISTS,ALBUMS,TRACKS".to_string());

        let res: TidalSearchResults = self
            .request(
                reqwest::Method::GET,
                "search",
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let favorites = self.favorite_ids.read().await;
        let tracks = res
            .tracks
            .map(|p| {
                p.items
                    .iter()
                    .map(|t| Self::map_track(t, &favorites))
                    .collect()
            })
            .unwrap_or_default();
        let albums = res
            .albums
            .map(|p| p.items.iter().map(Self::map_album).collect::<Vec<_>>())
            .unwrap_or_default();

        for album in &albums {
            self.cache
                .insert(
                    format!("album:{}", album.id),
                    CachedItem::SingleAlbum(album.clone()),
                )
                .await;
        }

        let artists = res
            .artists
            .map(|p| p.items.iter().map(Self::map_artist).collect())
            .unwrap_or_default();

        let result = UnifiedSearchResult {
            tracks,
            albums,
            artists,
        };
        self.cache
            .insert(key, CachedItem::SearchResult(result.clone()))
            .await;
        Ok(result)
    }

    async fn get_artist(&self, id: &str) -> Result<Artist, String> {
        let key = format!("artist:{}", id);
        if let Some(CachedItem::SingleArtist(a)) = self.cache.get(&key).await {
            return Ok(a);
        }
        let res: TidalArtist = self
            .request(
                reqwest::Method::GET,
                &format!("artists/{}", id),
                None,
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;
        let artist = Self::map_artist(&res);
        self.cache
            .insert(key, CachedItem::SingleArtist(artist.clone()))
            .await;
        Ok(artist)
    }

    async fn get_track(&self, track_id: &str) -> Result<Track, String> {
        let key = format!("track:{}", track_id);
        if let Some(CachedItem::SingleTrack(t)) = self.cache.get(&key).await {
            return Ok(t);
        }
        let res: TidalTrack = self
            .request(
                reqwest::Method::GET,
                &format!("tracks/{}", track_id),
                None,
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;
        let favorites = self.favorite_ids.read().await;
        let track = Self::map_track(&res, &favorites);
        self.cache
            .insert(key, CachedItem::SingleTrack(track.clone()))
            .await;
        Ok(track)
    }

    async fn get_album(&self, album_id: &str) -> Result<Album, String> {
        let key = format!("album:{}", album_id);
        if let Some(CachedItem::SingleAlbum(a)) = self.cache.get(&key).await {
            return Ok(a);
        }
        let res: TidalAlbum = self
            .request(
                reqwest::Method::GET,
                &format!("albums/{}", album_id),
                None,
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;
        let album = Self::map_album(&res);
        self.cache
            .insert(key, CachedItem::SingleAlbum(album.clone()))
            .await;
        Ok(album)
    }

    async fn set_track_liked(&self, track_id: &str, liked: bool) -> Result<(), String> {
        let user_id = {
            let creds = self.credentials.read().await;
            creds.user_id.clone().ok_or("No user ID")?
        };

        let method = if liked {
            reqwest::Method::POST
        } else {
            reqwest::Method::DELETE
        };
        let body = if liked {
            Some(serde_json::json!({"trackIds": track_id}))
        } else {
            None
        };

        let path = format!(
            "users/{}/favorites/tracks/{}",
            user_id,
            if liked { "" } else { track_id }
        );

        let _: serde_json::Value = self
            .request(method, &path, None, body, ApiVersion::V1)
            .await
            .map_err(|e| e.to_string())?;

        let key = format!("favorites:{}", user_id);
        self.cache.invalidate(&key).await;

        Ok(())
    }

    async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        let user_id = {
            let creds = self.credentials.read().await;
            creds.user_id.clone().ok_or("No user ID")?
        };

        let key = format!("playlists:{}", user_id);
        if let Some(CachedItem::PlaylistList(p)) = self.cache.get(&key).await {
            return Ok(p);
        }

        let res: TidalPage<TidalPlaylist> = self
            .request(
                reqwest::Method::GET,
                &format!("users/{}/playlists", user_id),
                None,
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let playlists: Vec<Playlist> = res.items.iter().map(Self::map_playlist).collect();
        self.cache
            .insert(key, CachedItem::PlaylistList(playlists.clone()))
            .await;
        Ok(playlists)
    }

    async fn create_playlist(&self, name: &str) -> Result<Playlist, String> {
        let user_id = {
            let creds = self.credentials.read().await;
            creds.user_id.clone().ok_or("No user ID")?
        };

        let params = HashMap::from([("name".to_string(), name.to_string())]);
        let res: TidalPlaylist = self
            .request(
                reqwest::Method::POST,
                &format!("users/{}/playlists", user_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let key = format!("playlists:{}", user_id);
        self.cache.invalidate(&key).await;

        Ok(Self::map_playlist(&res))
    }

    async fn delete_playlist(&self, id: &str) -> Result<(), String> {
        let _: serde_json::Value = self
            .request(
                reqwest::Method::DELETE,
                &format!("playlists/{}", id),
                None,
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn add_to_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        let params = HashMap::from([("trackIds".to_string(), track_id.to_string())]);
        let _: serde_json::Value = self
            .request(
                reqwest::Method::POST,
                &format!("playlists/{}/tracks", playlist_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn remove_from_playlist(
        &self,
        _playlist_id: &str,
        _track_id: &str,
    ) -> Result<(), String> {
        Err("soon".to_string())
    }

    async fn resolve_stream(&self, track_id: &str) -> Result<AudioStream, String> {
        let mut params = HashMap::new();
        params.insert("audioquality".to_string(), "HI_RES_LOSSLESS".to_string());
        params.insert("playbackmode".to_string(), "STREAM".to_string());
        params.insert("assetpresentation".to_string(), "FULL".to_string());

        let info: TidalPlaybackInfo = self
            .request(
                reqwest::Method::GET,
                &format!("tracks/{}/playbackinfo", track_id),
                Some(params),
                None,
                ApiVersion::Desktop,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(AudioStream::Url("https://example.com".to_string()))
    }

    async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, String> {
        let key = format!("playlist_tracks:{}", playlist_id);
        if let Some(CachedItem::TrackList(t)) = self.cache.get(&key).await {
            return Ok(t);
        }

        let params = HashMap::from([("limit".to_string(), "100".to_string())]);
        let res: TidalPage<TidalTrack> = self
            .request(
                reqwest::Method::GET,
                &format!("playlists/{}/tracks", playlist_id),
                Some(params),
                None,
                ApiVersion::V1,
            )
            .await
            .map_err(|e| e.to_string())?;

        let favorites = self.favorite_ids.read().await;
        let tracks: Vec<Track> = res
            .items
            .iter()
            .map(|t| Self::map_track(t, &favorites))
            .collect();
        self.cache
            .insert(key, CachedItem::TrackList(tracks.clone()))
            .await;
        Ok(tracks)
    }
}
