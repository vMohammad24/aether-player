use crate::models::entities::{Album, Artist, Genre, Playlist, Track, UnifiedSearchResult};
use crate::traits::{AudioStream, LibraryProvider};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use log::info;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const TIDAL_AUTH_BASE_URI: &str = "https://auth.tidal.com/v1";
const TIDAL_API_URL: &str = "https://api.tidal.com/v1";
const TIDAL_API_V2_URL: &str = "https://api.tidal.com/v2";
const TIDAL_OPENAPI_V2_URL: &str = "https://openapi.tidal.com/v2";
const TIDAL_DESKTOP_V1_URL: &str = "https://desktop.tidal.com/v1";
const TIDAL_DESKTOP_V2_URL: &str = "https://desktop.tidal.com/v2";
const TIDAL_VERSION: &str = "2026.1.5";
const TIDAL_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) TIDAL/1.8.0-beta Chrome/126.0.6478.127 Electron/31.2.1 Safari/537.36";
const TIDAL_CLIENT_ID: &str = env!("TIDAL_CLIENT_ID");
const TIDAL_CLIENT_SECRET: &str = env!("TIDAL_CLIENT_SECRET");
const TIDAL_TOKEN: &str = env!("TIDAL_CLIENT_TOKEN");

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

    fn requires_oauth(&self) -> bool {
        !matches!(self, ApiVersion::V1)
    }
}

#[derive(Clone)]
pub struct TidalProvider {
    id: String,
    name: String,
    credentials: Arc<RwLock<TidalCredentials>>,
    client: Client,
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

impl TidalProvider {
    pub fn new(id: String, name: String, credentials: TidalCredentials) -> Result<Self> {
        Ok(Self {
            id,
            name,
            credentials: Arc::new(RwLock::new(credentials)),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent(TIDAL_USER_AGENT)
                .build()
                .context("Failed to build HTTP client")?,
        })
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
            let mut params = HashMap::new();
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
            .ok_or_else(|| anyhow!("OAuth token required but not available"))?;

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
        query_params.insert("locale".to_string(), "en_US".to_string());
        query_params.insert("platform".to_string(), "DESKTOP".to_string());

        let mut request = self
            .client
            .request(method.clone(), &url)
            .query(&query_params)
            .header("user-agent", TIDAL_USER_AGENT)
            .header("x-tidal-client-version", TIDAL_VERSION)
            .header("Authorization", format!("Bearer {}", access_token));

        request = match api_version {
            ApiVersion::V2 => request.header("Accept", "application/vnd.tidal.v1+json"),
            ApiVersion::OpenApi | ApiVersion::Desktop | ApiVersion::DesktopV2 => {
                request.header("Accept", "application/json")
            }
            ApiVersion::V1 => request,
        };

        if let Some(body) = data {
            request = request.json(&body);
        }

        let response = request.send().await.context("Failed to send request")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        response
            .json::<T>()
            .await
            .context("Failed to parse response JSON")
    }

    pub async fn start_device_auth() -> Result<DeviceAuthPending> {
        let client = Client::new();
        let scopes = "r_usr w_usr w_sub";

        let mut params = HashMap::new();
        params.insert("client_id", TIDAL_CLIENT_ID);
        params.insert("client_secret", TIDAL_CLIENT_SECRET);
        params.insert("scope", scopes);

        let res = client
            .post(format!(
                "{}/oauth2/device_authorization",
                TIDAL_AUTH_BASE_URI
            ))
            .form(&params)
            .send()
            .await?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        info!("Device auth response status: {}, Data: {}", status, text);
        if !status.is_success() {
            return Err(anyhow!("Device auth init failed: {}", text));
        }

        let data: DeviceAuthResponse = serde_json::from_str(&text)?;

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
            }

            let status = res.status();
            if status == StatusCode::BAD_REQUEST {
                let err_body = res.text().await?;
                if err_body.contains("authorization_pending") {
                    tokio::time::sleep(std::time::Duration::from_secs(pending.interval as u64))
                        .await;
                    continue;
                }
                return Err(anyhow!("Auth failed: {}", err_body));
            }

            return Err(anyhow!("Unexpected status: {}", status));
        }
    }
}

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

#[async_trait::async_trait]
impl LibraryProvider for TidalProvider {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    async fn get_artist_albums(&self, _artist_id: &str) -> Result<Vec<Album>, String> {
        Ok(Vec::new())
    }

    async fn get_album_tracks(&self, _album_id: &str) -> Result<Vec<Track>, String> {
        Ok(Vec::new())
    }

    async fn get_recent_albums(&self, _limit: u32) -> Result<Vec<Album>, String> {
        Ok(Vec::new())
    }

    async fn get_random_albums(&self, _limit: u32) -> Result<Vec<Album>, String> {
        Ok(Vec::new())
    }

    async fn get_most_played_tracks(&self, _limit: u32) -> Result<Vec<Track>, String> {
        Ok(Vec::new())
    }

    async fn get_library_stats(&self) -> Result<crate::models::entities::LibraryStats, String> {
        Err("Not implemented".to_string())
    }

    async fn get_genres(&self) -> Result<Vec<Genre>, String> {
        Ok(Vec::new())
    }

    async fn get_genre_tracks(&self, _genre: &str) -> Result<Vec<Track>, String> {
        Ok(Vec::new())
    }

    async fn get_favorites(&self) -> Result<Vec<Track>, String> {
        Ok(Vec::new())
    }

    async fn search(&self, _query: &str) -> Result<UnifiedSearchResult, String> {
        Err("Not implemented".to_string())
    }

    async fn get_artist(&self, _id: &str) -> Result<Artist, String> {
        Err("Not implemented".to_string())
    }

    async fn get_track(&self, _track_id: &str) -> Result<Track, String> {
        Err("Not implemented".to_string())
    }

    async fn get_album(&self, _album_id: &str) -> Result<Album, String> {
        Err("Not implemented".to_string())
    }

    async fn set_track_liked(&self, _track_id: &str, _liked: bool) -> Result<(), String> {
        Ok(())
    }

    async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        Ok(Vec::new())
    }

    async fn create_playlist(&self, _name: &str) -> Result<Playlist, String> {
        Err("Not implemented".to_string())
    }

    async fn delete_playlist(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn add_to_playlist(&self, _playlist_id: &str, _track_id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn remove_from_playlist(
        &self,
        _playlist_id: &str,
        _track_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn resolve_stream(&self, _track_id: &str) -> Result<AudioStream, String> {
        Err("Not implemented".to_string())
    }

    async fn scan(&self) -> Result<(), String> {
        Ok(())
    }

    async fn get_playlist_tracks(&self, _playlist_id: &str) -> Result<Vec<Track>, String> {
        Ok(Vec::new())
    }
}
