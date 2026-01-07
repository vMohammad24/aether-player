use crate::models::entities::{
    Album, Artist, Genre, LibraryStats, Playlist, Track, UnifiedSearchResult,
};
use crate::traits::{AudioStream, LibraryProvider};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use moka::future::Cache;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone)]
pub struct SubsonicProvider {
    id: String,
    name: String,
    base_url: String,
    username: String,
    token: String,
    salt: String,
    client: Client,

    cache: Cache<String, String>,
}

impl SubsonicProvider {
    pub fn new(
        id: String,
        name: String,
        url: String,
        username: String,
        token: String,
        salt: String,
    ) -> Result<Self> {
        let cache = Cache::builder()
            .max_capacity(500)
            .time_to_live(Duration::from_secs(60 * 10))
            .build();

        Ok(Self {
            id,
            name,
            base_url: url.trim_end_matches('/').to_string(),
            username,
            token,
            salt,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .context("Failed to build HTTP client")?,
            cache,
        })
    }

    fn build_url(&self, endpoint: &str) -> String {
        format!(
            "{}/rest/{}?u={}&t={}&s={}&v=1.16.1&c=aether&f=json",
            self.base_url, endpoint, self.username, self.token, self.salt
        )
    }

    fn should_cache(&self, endpoint: &str) -> bool {
        match endpoint {
            "star" | "unstar" | "scrobble" | "startScan" | "getScanStatus" | "getRandomSongs"
            | "stream" => false,

            _ => true,
        }
    }

    fn get_cover_art_url(&self, id: &str) -> String {
        self.build_url("getCoverArt") + &format!("&id={}&size=600", id)
    }

    fn map_err(e: anyhow::Error) -> String {
        e.to_string()
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let mut url_str = self.build_url(endpoint);
        for (key, value) in query {
            url_str.push_str(&format!("&{}={}", key, value));
        }

        let use_cache = self.should_cache(endpoint);

        if use_cache {
            if let Some(cached_body) = self.cache.get(&url_str).await {
                if let Ok(response) = serde_json::from_str::<SubsonicResponse<T>>(&cached_body) {
                    if response.response.status != "failed" {
                        return Ok(response.response.content);
                    }
                }
            }
        }

        let url = Url::parse(&url_str).context("Invalid URL constructed")?;
        let res = self.client.get(url).send().await?;

        if !res.status().is_success() {
            let status = res.status();
            return Err(anyhow!("Subsonic request failed: HTTP {}", status));
        }

        let body = res.text().await?;

        let response: SubsonicResponse<T> = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse JSON from {}", endpoint))?;

        if response.response.status == "failed" {
            let err_msg = response
                .response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown API error".to_string());
            return Err(anyhow!("Subsonic API Error: {}", err_msg));
        }

        if use_cache {
            self.cache.insert(url_str, body).await;
        }

        Ok(response.response.content)
    }

    fn map_album(&self, sub: SubsonicAlbum) -> Album {
        Album {
            id: sub.id,
            title: sub.title,
            artist_id: sub.artist_id.or(sub.parent_id).unwrap_or_default(),
            artist_name: sub.artist.unwrap_or_default(),
            cover_art: sub.cover_art.map(|id| self.get_cover_art_url(&id)),
            year: sub.year.map(|y| y as u16),
            track_count: sub.song_count,
        }
    }

    fn map_track(&self, sub: SubsonicSong) -> Track {
        Track {
            id: sub.id,
            provider_id: Some(self.id.clone()),
            title: sub.title,
            artist_id: sub.artist_id.unwrap_or_default(),
            artist_name: sub.artist.unwrap_or_default(),
            album_id: sub.album_id.or(sub.parent_id).unwrap_or_default(),
            album_title: sub.album.unwrap_or_default(),
            duration_sec: sub.duration.unwrap_or(0),
            track_number: sub.track,
            disc_number: sub.disc_number,
            year: sub.year.map(|y| y as u16),
            genre: sub.genre,
            bitrate: sub.bitrate,
            play_count: sub.play_count.unwrap_or(0),
            liked: sub.starred.is_some(),
        }
    }

    fn map_playlist(&self, sub: SubsonicPlaylist) -> Playlist {
        let created_at = sub
            .created
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or_else(|| Utc::now());

        Playlist {
            id: sub.id,
            name: sub.name,
            track_count: sub.song_count,
            cover_art: sub.cover_art.map(|id| self.get_cover_art_url(&id)),
            owner: sub.owner.unwrap_or_else(|| "Unknown".to_string()),
            created_at,
        }
    }
}

#[async_trait]
impl LibraryProvider for SubsonicProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn get_recent_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let limit_str = limit.to_string();
        let res: AlbumList2Response = self
            .request("getAlbumList2", &[("type", "newest"), ("size", &limit_str)])
            .await
            .map_err(Self::map_err)?;

        let albums: Vec<Album> = res
            .album_list
            .album
            .unwrap_or_default()
            .into_iter()
            .map(|a| self.map_album(a))
            .collect();

        Ok(albums)
    }

    async fn get_random_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let limit_str = limit.to_string();

        let res: AlbumList2Response = self
            .request("getAlbumList2", &[("type", "random"), ("size", &limit_str)])
            .await
            .map_err(Self::map_err)?;

        let albums: Vec<Album> = res
            .album_list
            .album
            .unwrap_or_default()
            .into_iter()
            .map(|a| self.map_album(a))
            .collect();

        Ok(albums)
    }

    async fn get_most_played_tracks(&self, limit: u32) -> Result<Vec<Track>, String> {
        let res: AlbumList2Response = self
            .request("getAlbumList2", &[("type", "frequent"), ("size", "5")])
            .await
            .map_err(Self::map_err)?;

        let albums = res.album_list.album.unwrap_or_default();
        let mut all_tracks = Vec::new();

        for album in albums {
            if all_tracks.len() >= limit as usize {
                break;
            }
            if let Ok(album_details) = self.get_album_tracks(&album.id).await {
                all_tracks.extend(album_details);
            }
        }

        if all_tracks.is_empty() {
            let random_res: SongsContainerResponse = self
                .request("getRandomSongs", &[("size", &limit.to_string())])
                .await
                .map_err(Self::map_err)?;

            if let Some(songs) = random_res.songs.song {
                for s in songs {
                    all_tracks.push(self.map_track(s));
                }
            }
        }

        all_tracks.truncate(limit as usize);
        Ok(all_tracks)
    }

    async fn get_library_stats(&self) -> Result<LibraryStats, String> {
        let res: ScanStatusResponse = self
            .request("getScanStatus", &[])
            .await
            .map_err(Self::map_err)?;

        let count = res.scan_status.count.unwrap_or(0);

        Ok(LibraryStats {
            track_count: count,
            album_count: 0,
            artist_count: 0,
            total_duration: 0,
            average_bitrate: 0,
        })
    }

    async fn get_genres(&self) -> Result<Vec<Genre>, String> {
        let res: GetGenresResponse = self
            .request("getGenres", &[])
            .await
            .map_err(Self::map_err)?;

        let mut genres: Vec<Genre> = res
            .genres
            .genre
            .unwrap_or_default()
            .into_iter()
            .map(|g| Genre {
                name: g.value,
                track_count: g.song_count,
            })
            .collect();

        genres.sort_by(|a, b| b.track_count.cmp(&a.track_count));
        Ok(genres)
    }

    async fn get_genre_tracks(&self, genre: &str) -> Result<Vec<Track>, String> {
        let res: SongsByGenreResponse = self
            .request("getSongsByGenre", &[("genre", genre), ("count", "50")])
            .await
            .map_err(Self::map_err)?;

        let tracks: Vec<Track> = res
            .songs
            .song
            .unwrap_or_default()
            .into_iter()
            .map(|s| self.map_track(s))
            .collect();
        Ok(tracks)
    }

    async fn get_favorites(&self) -> Result<Vec<Track>, String> {
        let res: GetStarredResponse = self
            .request("getStarred", &[])
            .await
            .map_err(Self::map_err)?;

        let tracks: Vec<Track> = res
            .starred
            .song
            .unwrap_or_default()
            .into_iter()
            .map(|s| self.map_track(s))
            .collect();
        Ok(tracks)
    }

    async fn search(&self, query: &str) -> Result<UnifiedSearchResult, String> {
        let res: Search3Response = self
            .request(
                "search3",
                &[
                    ("query", query),
                    ("songCount", "20"),
                    ("albumCount", "20"),
                    ("artistCount", "20"),
                ],
            )
            .await
            .map_err(Self::map_err)?;

        let mut result = UnifiedSearchResult::default();

        if let Some(songs) = res.result.song {
            result.tracks = songs.into_iter().map(|s| self.map_track(s)).collect();
        }
        if let Some(albums) = res.result.album {
            result.albums = albums.into_iter().map(|a| self.map_album(a)).collect();
        }
        if let Some(artists) = res.result.artist {
            result.artists = artists
                .into_iter()
                .map(|a| Artist {
                    id: a.id.clone(),
                    name: a.name,
                    bio: None,
                    image_url: a.cover_art.map(|id| self.get_cover_art_url(&id)),
                })
                .collect();
        }
        Ok(result)
    }

    async fn get_artist(&self, id: &str) -> Result<Artist, String> {
        let artist_res: GetArtistResponse = self
            .request("getArtist", &[("id", id)])
            .await
            .map_err(Self::map_err)?;

        let info_res: Result<GetArtistInfoResponse> =
            self.request("getArtistInfo", &[("id", id)]).await;

        let mut artist = Artist {
            id: artist_res.artist.id,
            name: artist_res.artist.name,
            bio: None,
            image_url: artist_res
                .artist
                .cover_art
                .map(|cid| self.get_cover_art_url(&cid)),
        };

        if let Ok(info) = info_res {
            artist.bio = info.artist_info.biography;
            if let Some(url) = info.artist_info.large_image_url {
                if !url.is_empty() {
                    artist.image_url = Some(url);
                }
            }
        }

        Ok(artist)
    }

    async fn get_album(&self, id: &str) -> Result<Album, String> {
        let res: GetAlbumResponse = self
            .request("getAlbum", &[("id", id)])
            .await
            .map_err(Self::map_err)?;
        Ok(self.map_album(res.album.info))
    }

    async fn get_artist_albums(&self, artist_id: &str) -> Result<Vec<Album>, String> {
        let res: GetArtistResponse = self
            .request("getArtist", &[("id", artist_id)])
            .await
            .map_err(Self::map_err)?;

        let albums = res
            .artist
            .album
            .unwrap_or_default()
            .into_iter()
            .map(|a| self.map_album(a))
            .collect();
        Ok(albums)
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Track>, String> {
        let res: GetAlbumResponse = self
            .request("getAlbum", &[("id", album_id)])
            .await
            .map_err(Self::map_err)?;

        let tracks: Vec<Track> = res
            .album
            .song
            .unwrap_or_default()
            .into_iter()
            .map(|s| self.map_track(s))
            .collect();

        Ok(tracks)
    }

    async fn get_track(&self, track_id: &str) -> Result<Track, String> {
        let res: GetSongResponse = self
            .request("getSong", &[("id", track_id)])
            .await
            .map_err(Self::map_err)?;
        Ok(self.map_track(res.song))
    }

    async fn set_track_liked(&self, track_id: &str, liked: bool) -> Result<(), String> {
        let endpoint = if liked { "star" } else { "unstar" };

        let _: serde_json::Value = self
            .request(endpoint, &[("id", track_id)])
            .await
            .map_err(Self::map_err)?;

        let starred_url = self.build_url("getStarred");
        self.cache.remove(&starred_url).await;

        let mut song_url = self.build_url("getSong");
        song_url.push_str(&format!("&id={}", track_id));
        self.cache.remove(&song_url).await;

        Ok(())
    }

    async fn resolve_stream(&self, track_id: &str) -> Result<AudioStream, String> {
        let stream_url = self.build_url("stream") + &format!("&id={}", track_id);
        Ok(AudioStream::Url(stream_url))
    }

    async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        let res: GetPlaylistsResponse = self
            .request("getPlaylists", &[])
            .await
            .map_err(Self::map_err)?;

        let playlists: Vec<Playlist> = res
            .playlists
            .playlist
            .unwrap_or_default()
            .into_iter()
            .map(|p| self.map_playlist(p))
            .collect();

        Ok(playlists)
    }

    async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, String> {
        let res: GetPlaylistResponse = self
            .request("getPlaylist", &[("id", playlist_id)])
            .await
            .map_err(Self::map_err)?;

        let tracks: Vec<Track> = res
            .playlist
            .entry
            .unwrap_or_default()
            .into_iter()
            .map(|s| self.map_track(s))
            .collect();

        Ok(tracks)
    }

    async fn scan(&self) -> Result<(), String> {
        self.cache.invalidate_all();
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    response: SubsonicResponseContent<T>,
}

#[derive(Debug, Deserialize)]
struct SubsonicResponseContent<T> {
    status: String,
    #[serde(flatten)]
    content: T,
    error: Option<SubsonicError>,
}

#[derive(Debug, Deserialize)]
struct SubsonicError {
    #[allow(dead_code)]
    code: i32,
    message: String,
}

#[derive(Deserialize)]
struct GetArtistResponse {
    artist: SubsonicArtistDetail,
}

#[derive(Deserialize)]
struct SubsonicArtistDetail {
    id: String,
    name: String,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    album: Option<Vec<SubsonicAlbum>>,
}

#[derive(Deserialize)]
struct GetArtistInfoResponse {
    #[serde(rename = "artistInfo")]
    artist_info: SubsonicArtistInfo,
}

#[derive(Deserialize)]
struct SubsonicArtistInfo {
    #[serde(rename = "biography")]
    biography: Option<String>,
    #[serde(rename = "largeImageUrl")]
    large_image_url: Option<String>,
}

#[derive(Deserialize)]
struct SubsonicAlbum {
    id: String,
    #[serde(rename = "parent")]
    parent_id: Option<String>,
    #[serde(alias = "name")]
    title: String,
    artist: Option<String>,
    #[serde(rename = "artistId")]
    artist_id: Option<String>,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    year: Option<i32>,
    #[serde(rename = "songCount")]
    song_count: Option<u32>,
}

#[derive(Deserialize)]
struct GetAlbumResponse {
    album: SubsonicAlbumDetail,
}

#[derive(Deserialize)]
struct SubsonicAlbumDetail {
    #[serde(flatten)]
    info: SubsonicAlbum,
    song: Option<Vec<SubsonicSong>>,
}

#[derive(Deserialize)]
struct AlbumList2Response {
    #[serde(rename = "albumList2")]
    album_list: AlbumListContainer,
}

#[derive(Deserialize)]
struct AlbumListContainer {
    album: Option<Vec<SubsonicAlbum>>,
}

#[derive(Deserialize)]
struct SubsonicSong {
    id: String,
    #[serde(rename = "parent")]
    parent_id: Option<String>,
    title: String,
    album: Option<String>,
    #[serde(rename = "albumId")]
    album_id: Option<String>,
    artist: Option<String>,
    #[serde(rename = "artistId")]
    artist_id: Option<String>,
    year: Option<i32>,
    track: Option<u32>,
    #[serde(rename = "discNumber")]
    disc_number: Option<u32>,
    genre: Option<String>,
    duration: Option<u32>,
    #[serde(rename = "bitRate")]
    bitrate: Option<u32>,
    #[serde(rename = "playCount")]
    play_count: Option<u32>,
    starred: Option<String>,
}

#[derive(Deserialize)]
struct GetSongResponse {
    song: SubsonicSong,
}

#[derive(Deserialize)]
struct SongsByGenreResponse {
    #[serde(rename = "songsByGenre")]
    songs: SongsContainer,
}

#[derive(Deserialize)]
struct SongsContainerResponse {
    #[serde(rename = "randomSongs")]
    songs: SongsContainer,
}

#[derive(Deserialize)]
struct SongsContainer {
    song: Option<Vec<SubsonicSong>>,
}

#[derive(Deserialize)]
struct GetPlaylistsResponse {
    playlists: PlaylistsContainer,
}

#[derive(Deserialize)]
struct PlaylistsContainer {
    playlist: Option<Vec<SubsonicPlaylist>>,
}

#[derive(Deserialize)]
struct SubsonicPlaylist {
    id: String,
    name: String,
    #[serde(rename = "songCount")]
    song_count: u32,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
    owner: Option<String>,
    created: Option<String>,
}

#[derive(Deserialize)]
struct GetPlaylistResponse {
    playlist: SubsonicPlaylistDetail,
}

#[derive(Deserialize)]
struct SubsonicPlaylistDetail {
    #[serde(flatten)]
    _info: SubsonicPlaylist,
    entry: Option<Vec<SubsonicSong>>,
}

#[derive(Deserialize)]
struct Search3Response {
    #[serde(rename = "searchResult3")]
    result: SearchResultContainer,
}

#[derive(Deserialize)]
struct SearchResultContainer {
    artist: Option<Vec<SubsonicArtistID3>>,
    album: Option<Vec<SubsonicAlbum>>,
    song: Option<Vec<SubsonicSong>>,
}

#[derive(Deserialize)]
struct SubsonicArtistID3 {
    id: String,
    name: String,
    #[serde(rename = "coverArt")]
    cover_art: Option<String>,
}

#[derive(Deserialize)]
struct GetStarredResponse {
    starred: StarredContainer,
}

#[derive(Deserialize)]
struct StarredContainer {
    song: Option<Vec<SubsonicSong>>,
}

#[derive(Deserialize)]
struct GetGenresResponse {
    genres: GenresContainer,
}

#[derive(Deserialize)]
struct GenresContainer {
    genre: Option<Vec<SubsonicGenre>>,
}

#[derive(Deserialize)]
struct SubsonicGenre {
    value: String,
    #[serde(rename = "songCount")]
    song_count: u32,
}

#[derive(Deserialize)]
struct ScanStatusResponse {
    #[serde(rename = "scanStatus")]
    scan_status: SubsonicScanStatus,
}

#[derive(Deserialize)]
struct SubsonicScanStatus {
    count: Option<u32>,
}
