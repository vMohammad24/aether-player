use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_ROOT: &str = "http://ws.audioscrobbler.com/2.0/";

#[derive(Clone)]
pub struct LastFmClient {
    api_key: String,
    api_secret: String,
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

impl LastFmClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
            session_key: None,
            client: Client::new(),
        }
    }

    pub fn set_session_key(&mut self, session_key: String) {
        self.session_key = Some(session_key);
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
        sig_base.push_str(&self.api_secret);

        let digest = md5::compute(sig_base);
        params.insert("api_sig".to_string(), format!("{:x}", digest));
    }

    pub async fn get_artist_info(&self, artist: &str) -> Result<ArtistInfo> {
        let params = [
            ("method", "artist.getInfo"),
            ("artist", artist),
            ("api_key", &self.api_key),
            ("format", "json"),
            ("autocorrect", "1"),
        ];

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

        let data: ArtistInfoResponse = res.json().await.context("Failed to parse Last.fm response")?;
        Ok(data.artist)
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
        params.insert("api_key".to_string(), self.api_key.clone());
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
            return Err(anyhow::anyhow!("Last.fm Scrobble Error {}: {}", status, text));
        }

        Ok(())
    }
}
