use crate::models::entities::{Album, Artist, Genre, Playlist, Track, UnifiedSearchResult};
use crate::traits::{AudioStream, LibraryProvider};
use crate::util::lastfm::LastFmClient;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;
use jwalk::WalkDir;
use lofty::prelude::*;
use lofty::read_from_path;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::sync::mpsc;

const BATCH_SIZE: usize = 200;
const COVERS_DIR: &str = "covers";

use crate::models::AppConfig;

pub struct LocalProvider {
    db: SqlitePool,
    id: String,
    data_dir: PathBuf,
    config: AppConfig,
}

impl LocalProvider {
    pub async fn new(
        id: String,
        db_path: &Path,
        data_dir: &Path,
        config: AppConfig,
    ) -> Result<Self> {
        if !data_dir.exists() {
            fs::create_dir_all(data_dir).context("Failed to create data directory")?;
        }

        let covers_path = data_dir.join(COVERS_DIR);
        if !covers_path.exists() {
            fs::create_dir_all(&covers_path).context("Failed to create covers directory")?;
        }

        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Failed to create database directory")?;
            }
        }

        if !db_path.exists() {
            fs::File::create(db_path).context("Failed to create database file")?;
        }

        let db_url = format!("sqlite://{}", db_path.to_string_lossy());
        let db = SqlitePool::connect(&db_url).await?;

        let provider = Self {
            db,
            id,
            data_dir: data_dir.to_path_buf(),
            config,
        };

        provider.init_schema().await?;
        Ok(provider)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            "PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA temp_store = MEMORY;",
        )
        .execute(&self.db)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                bio TEXT,
                image_url TEXT,
                UNIQUE(name)
            );

            CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id TEXT,
                cover_art TEXT,
                year INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(artist_id) REFERENCES artists(id),
                UNIQUE(title, artist_id)
            );

            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                title TEXT NOT NULL,
                artist_id TEXT,
                album_id TEXT,
                duration_sec INTEGER,
                track_number INTEGER,
                disc_number INTEGER,
                year INTEGER,
                genre TEXT,
                bitrate INTEGER,
                play_count INTEGER DEFAULT 0,
                liked BOOLEAN DEFAULT 0,
                mtime INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(artist_id) REFERENCES artists(id),
                FOREIGN KEY(album_id) REFERENCES albums(id),
                UNIQUE(path)
            );

            CREATE TABLE IF NOT EXISTS track_artists (
                track_id TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                PRIMARY KEY(track_id, artist_id),
                FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE,
                FOREIGN KEY(artist_id) REFERENCES artists(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS album_artists (
                album_id TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                PRIMARY KEY(album_id, artist_id),
                FOREIGN KEY(album_id) REFERENCES albums(id) ON DELETE CASCADE,
                FOREIGN KEY(artist_id) REFERENCES artists(id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS library_roots (
                path TEXT PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                owner TEXT DEFAULT 'local',
                cover_art TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT,
                track_id TEXT,
                position INTEGER,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album_id);
            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist_id);
            CREATE INDEX IF NOT EXISTS idx_track_artists_artist ON track_artists(artist_id);
            CREATE INDEX IF NOT EXISTS idx_album_artists_artist ON album_artists(artist_id);
            CREATE INDEX IF NOT EXISTS idx_albums_artist ON albums(artist_id);
            CREATE INDEX IF NOT EXISTS idx_tracks_liked ON tracks(liked) WHERE liked = 1;

            CREATE TABLE IF NOT EXISTS scan_found (
                path TEXT PRIMARY KEY
            );
            "#,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn scan_path(
        &self,
        root_path: &str,
        existing_map: Arc<HashMap<PathBuf, i64>>,
    ) -> Result<()> {
        let root = root_path.to_string();
        let db = self.db.clone();
        let covers_dir = self.data_dir.join(COVERS_DIR);

        enum ScanResult {
            Found(PathBuf),
            New(PathBuf, ParsedMetadata, i64),
        }

        let (tx, mut rx) = mpsc::channel::<ScanResult>(200);

        log::info!("Starting scan of: {}", root);

        let consumer_handle = tokio::spawn(async move {
            let mut artist_cache: HashMap<String, String> = HashMap::new();
            let mut album_cache: HashMap<String, String> = HashMap::new();

            let mut pending_tracks = Vec::with_capacity(BATCH_SIZE);
            let mut pending_found = Vec::with_capacity(BATCH_SIZE);
            let mut processed_count = 0;

            while let Some(res) = rx.recv().await {
                processed_count += 1;

                match res {
                    ScanResult::Found(path) => {
                        pending_found.push(path);
                        if pending_found.len() >= BATCH_SIZE * 5 {
                            flush_found(&db, &mut pending_found).await;
                        }
                    }
                    ScanResult::New(path, meta, mtime) => {
                        let mut track_artist_ids = Vec::new();
                        for artist_name in &meta.artists {
                            if let Some(id) = artist_cache.get(artist_name) {
                                track_artist_ids.push(id.clone());
                            } else {
                                match resolve_artist_single(&db, artist_name).await {
                                    Ok(id) => {
                                        artist_cache.insert(artist_name.clone(), id.clone());
                                        track_artist_ids.push(id);
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Failed to resolve artist {}: {}",
                                            artist_name,
                                            e
                                        );
                                    }
                                }
                            }
                        }

                        if track_artist_ids.is_empty() {
                            let unknown_name = "Unknown Artist".to_string();
                            if let Ok(id) = resolve_artist_single(&db, &unknown_name).await {
                                track_artist_ids.push(id);
                            }
                        }

                        let primary_artist_id =
                            track_artist_ids.first().cloned().unwrap_or_default();

                        let album_artist_name =
                            meta.album_artist.as_ref().unwrap_or(&meta.artists[0]);
                        let album_artist_id =
                            match resolve_artist_single(&db, album_artist_name).await {
                                Ok(id) => id,
                                Err(_) => primary_artist_id.clone(),
                            };
                        let album_key = format!("{}::{}", album_artist_id, meta.album);

                        let album_id = if let Some(id) = album_cache.get(&album_key) {
                            id.clone()
                        } else {
                            match resolve_album(
                                &db,
                                &meta.album,
                                &album_artist_id,
                                &track_artist_ids,
                                &meta.cover_image,
                                &covers_dir,
                            )
                            .await
                            {
                                Ok(id) => {
                                    album_cache.insert(album_key.clone(), id.clone());
                                    id
                                }
                                Err(e) => {
                                    log::error!("Failed to resolve album {}: {}", meta.album, e);
                                    continue;
                                }
                            }
                        };

                        pending_found.push(path.clone());
                        pending_tracks.push((path, meta, track_artist_ids, album_id, mtime));

                        if pending_tracks.len() >= BATCH_SIZE {
                            flush_tracks(&db, &mut pending_tracks).await;
                        }
                        if pending_found.len() >= BATCH_SIZE * 5 {
                            flush_found(&db, &mut pending_found).await;
                        }
                    }
                }
            }

            if !pending_tracks.is_empty() {
                flush_tracks(&db, &mut pending_tracks).await;
            }
            if !pending_found.is_empty() {
                flush_found(&db, &mut pending_found).await;
            }

            log::info!("Scan complete. Processed {} items.", processed_count);
        });

        tokio::task::spawn_blocking(move || {
            let walker = WalkDir::new(&root).follow_links(true).into_iter();

            walker.par_bridge().for_each(|entry_res| match entry_res {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if [
                                "mp3", "flac", "wav", "m4a", "ogg", "opus", "aac", "alac", "aiff",
                            ]
                            .contains(&ext_str.as_str())
                            {
                                let mtime = entry
                                    .metadata()
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .map(|t| {
                                        t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                                            as i64
                                    })
                                    .unwrap_or(0);

                                if let Some(existing_mtime) = existing_map.get(&path) {
                                    if *existing_mtime == mtime {
                                        let _ =
                                            tx.blocking_send(ScanResult::Found(path.to_path_buf()));
                                        return;
                                    }
                                }

                                match parse_metadata(&path) {
                                    Ok(meta) => {
                                        if tx
                                            .blocking_send(ScanResult::New(
                                                path.to_path_buf(),
                                                meta,
                                                mtime,
                                            ))
                                            .is_err()
                                        {}
                                    }
                                    Err(e) => {
                                        log::warn!("Skipping {}: {}", path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("WalkDir error: {}", e);
                }
            });
        })
        .await?;

        consumer_handle.await.context("Consumer task failed")?;
        Ok(())
    }
}

async fn resolve_artist_single(db: &SqlitePool, name: &str) -> Result<String> {
    let name_trimmed = name.trim();

    let existing = sqlx::query("SELECT id, name FROM artists WHERE name = ? COLLATE NOCASE")
        .bind(name_trimmed)
        .fetch_optional(db)
        .await?;

    if let Some(row) = existing {
        return Ok(row.get("id"));
    }

    let new_id = uuid::Uuid::new_v4().to_string();

    let res = sqlx::query("INSERT OR IGNORE INTO artists (id, name) VALUES (?, ?)")
        .bind(&new_id)
        .bind(name_trimmed)
        .execute(db)
        .await?;

    if res.rows_affected() > 0 {
        Ok(new_id)
    } else {
        let row = sqlx::query("SELECT id FROM artists WHERE name = ? COLLATE NOCASE")
            .bind(name_trimmed)
            .fetch_one(db)
            .await?;
        Ok(row.get("id"))
    }
}

async fn resolve_album(
    db: &SqlitePool,
    title: &str,
    primary_artist_id: &str,
    all_artist_ids: &[String],
    cover_image: &Option<CoverImageData>,
    covers_dir: &Path,
) -> Result<String> {
    let existing = sqlx::query("SELECT id FROM albums WHERE title = ? AND artist_id = ?")
        .bind(title)
        .bind(primary_artist_id)
        .fetch_optional(db)
        .await?;

    let album_id = if let Some(row) = existing {
        row.get("id")
    } else {
        let new_id = uuid::Uuid::new_v4().to_string();
        let cover_path_str = if let Some(img_data) = cover_image {
            match save_cover_art(covers_dir, img_data) {
                Ok(p) => Some(p),
                Err(e) => {
                    log::warn!("Failed to save cover art for {}: {}", title, e);
                    None
                }
            }
        } else {
            None
        };

        let res = sqlx::query(
            "INSERT OR IGNORE INTO albums (id, title, artist_id, cover_art) VALUES (?, ?, ?, ?)",
        )
        .bind(&new_id)
        .bind(title)
        .bind(primary_artist_id)
        .bind(cover_path_str)
        .execute(db)
        .await?;

        if res.rows_affected() > 0 {
            new_id
        } else {
            let row = sqlx::query("SELECT id FROM albums WHERE title = ? AND artist_id = ?")
                .bind(title)
                .bind(primary_artist_id)
                .fetch_one(db)
                .await?;
            row.get("id")
        }
    };

    for artist_id in all_artist_ids {
        sqlx::query("INSERT OR IGNORE INTO album_artists (album_id, artist_id) VALUES (?, ?)")
            .bind(&album_id)
            .bind(artist_id)
            .execute(db)
            .await?;
    }

    Ok(album_id)
}

async fn flush_tracks(
    db: &SqlitePool,
    tracks: &mut Vec<(PathBuf, ParsedMetadata, Vec<String>, String, i64)>,
) {
    if tracks.is_empty() {
        return;
    }

    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to begin transaction: {}", e);
            return;
        }
    };

    for (path, meta, artist_ids, album_id, mtime) in tracks.iter() {
        let track_id = uuid::Uuid::new_v4().to_string();
        let path_str = path.to_string_lossy().to_string();
        let primary_artist = artist_ids.first().cloned().unwrap_or_default();

        let q = sqlx::query(
            r#"INSERT INTO tracks 
            (id, path, title, artist_id, album_id, duration_sec, track_number, disc_number, year, genre, bitrate, mtime) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
            title=excluded.title,
            artist_id=excluded.artist_id,
            album_id=excluded.album_id,
            duration_sec=excluded.duration_sec,
            track_number=excluded.track_number,
            disc_number=excluded.disc_number,
            year=excluded.year,
            genre=excluded.genre,
            bitrate=excluded.bitrate,
            mtime=excluded.mtime,
            created_at=CURRENT_TIMESTAMP
            "#
        )
        .bind(&track_id)
        .bind(&path_str)
        .bind(&meta.title)
        .bind(&primary_artist)
        .bind(album_id)
        .bind(meta.duration)
        .bind(meta.track_number)
        .bind(meta.disc_number)
        .bind(meta.year)
        .bind(&meta.genre)
        .bind(meta.bitrate)
        .bind(mtime);

        if let Err(e) = q.execute(&mut *tx).await {
            log::error!("Failed to insert track {}: {}", path_str, e);
            continue;
        }

        let actual_track_id = if sqlx::query("SELECT 1").execute(&mut *tx).await.is_ok() {
            match sqlx::query("SELECT id FROM tracks WHERE path = ?")
                .bind(&path_str)
                .fetch_one(&mut *tx)
                .await
            {
                Ok(row) => row.get::<String, _>("id"),
                Err(_) => track_id.clone(),
            }
        } else {
            track_id.clone()
        };

        let _ = sqlx::query("DELETE FROM track_artists WHERE track_id = ?")
            .bind(&actual_track_id)
            .execute(&mut *tx)
            .await;

        for artist_id in artist_ids {
            let jq = sqlx::query(
                "INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES (?, ?)",
            )
            .bind(&actual_track_id)
            .bind(artist_id);
            if let Err(e) = jq.execute(&mut *tx).await {
                log::error!("Failed to link artist to track: {}", e);
            }
        }
    }

    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit track batch: {}", e);
    }

    tracks.clear();
}

async fn flush_found(db: &SqlitePool, paths: &mut Vec<PathBuf>) {
    if paths.is_empty() {
        return;
    }
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!("Transaction error: {}", e);
            return;
        }
    };
    for path in paths.iter() {
        let path_str = path.to_string_lossy().to_string();
        let q = sqlx::query("INSERT OR IGNORE INTO scan_found (path) VALUES (?)").bind(path_str);
        let _ = q.execute(&mut *tx).await;
    }
    let _ = tx.commit().await;
    paths.clear();
}

fn save_cover_art(base_dir: &Path, img_data: &CoverImageData) -> Result<String> {
    let ext = match img_data.mime_type.as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        _ => "jpg",
    };
    let mut hasher = Sha256::new();
    hasher.update(&img_data.data);
    let hash_string = format!("{:x}", hasher.finalize());
    let filename = format!("{}.{}", hash_string, ext);
    let target_path = base_dir.join(&filename);
    if !target_path.exists() {
        fs::write(&target_path, &img_data.data)?;
    }
    Ok(filename)
}

struct CoverImageData {
    data: Vec<u8>,
    mime_type: String,
}

struct ParsedMetadata {
    title: String,
    artists: Vec<String>,
    album_artist: Option<String>,
    album: String,
    duration: u32,
    track_number: Option<u32>,
    disc_number: Option<u32>,
    year: Option<u16>,
    genre: Option<String>,
    bitrate: Option<u32>,
    cover_image: Option<CoverImageData>,
}

fn split_artists(raw: &str) -> Vec<String> {
    let raw = raw.replace(" feat. ", ";");
    let raw = raw.replace(" ft. ", ";");
    let raw = raw.replace(" & ", ";");
    let raw = raw.replace(" / ", ";");

    let raw = raw.replace(", ", ";");

    raw.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_metadata(path: &Path) -> Result<ParsedMetadata> {
    let tagged_file = read_from_path(path).map_err(|e| anyhow!("Lofty read error: {}", e))?;
    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as u32;
    let bitrate = properties.audio_bitrate();

    let filename_str = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut title = filename_str.clone();
    let mut artists = vec!["Unknown Artist".to_string()];
    let mut album = "Unknown Album".to_string();

    let mut track_number = None;
    let mut disc_number = None;
    let mut year = None;
    let mut genre = None;
    let mut cover_image = None;
    let mut album_artist = None;
    if let Some(tag) = tagged_file.primary_tag().or(tagged_file.first_tag()) {
        if let Some(t) = tag.title() {
            if !t.trim().is_empty() {
                title = t.trim().to_string();
            }
        }
        if let Some(a) = tag.artist() {
            if !a.trim().is_empty() {
                artists = split_artists(&a);
            }
        }
        if let Some(a) = tag.album() {
            if !a.trim().is_empty() {
                album = a.trim().to_string();
            }
        }
        track_number = tag.track();
        disc_number = tag.disk();
        if let Some(y) = tag.year() {
            year = Some(y as u16);
        }
        if let Some(g) = tag.genre() {
            if !g.trim().is_empty() {
                genre = Some(g.trim().to_string());
            }
        }
        if let Some(aa) = tag.get_string(&ItemKey::AlbumArtist) {
            if !aa.trim().is_empty() {
                album_artist = Some(aa.trim().to_string());
            }
        }

        let pictures = tag.pictures();
        if !pictures.is_empty() {
            let pic = pictures
                .iter()
                .find(|p| p.pic_type() == lofty::picture::PictureType::CoverFront)
                .or_else(|| pictures.first());
            if let Some(p) = pic {
                cover_image = Some(CoverImageData {
                    data: p.data().to_vec(),
                    mime_type: p
                        .mime_type()
                        .map(|m| m.to_string())
                        .unwrap_or("image/jpeg".to_string()),
                });
            }
        }
    }

    if artists.len() == 1 && artists[0] == "Unknown Artist" {
        let parts: Vec<&str> = filename_str.split(" - ").collect();
        match parts.len() {
            3 => {
                artists = split_artists(parts[0]);
                album = parts[1].trim().to_string();
                title = parts[2].trim().to_string();
            }
            2 => {
                artists = split_artists(parts[0]);
                title = parts[1].trim().to_string();
            }
            _ => {}
        }
    }

    Ok(ParsedMetadata {
        title,
        artists,
        album,
        album_artist,
        duration,
        track_number,
        disc_number,
        year,
        genre,
        bitrate,
        cover_image,
    })
}

#[async_trait]
impl LibraryProvider for LocalProvider {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        "Local Library"
    }

    async fn get_artist_albums(&self, artist_id: &str) -> Result<Vec<Album>, String> {
        let rows = sqlx::query(
            r#"SELECT DISTINCT al.id, al.title, al.artist_id, al.year, al.cover_art, 
                (SELECT name FROM artists WHERE id = al.artist_id) as artist_name,
                (SELECT COUNT(*) FROM tracks WHERE album_id = al.id) as track_count
            FROM albums al
            JOIN album_artists aa ON al.id = aa.album_id
            WHERE aa.artist_id = ? 
            ORDER BY al.year DESC"#,
        )
        .bind(artist_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(map_row_to_album).collect())
    }

    async fn get_album_tracks(&self, album_id: &str) -> Result<Vec<Track>, String> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as artist_name, al.title as album_title
            FROM tracks t
            LEFT JOIN artists a ON t.artist_id = a.id
            LEFT JOIN albums al ON t.album_id = al.id
            WHERE t.album_id = ? 
            ORDER BY t.disc_number ASC, t.track_number ASC"#,
        )
        .bind(album_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| map_row_to_track(r, Some(self.id.clone())))
            .collect())
    }

    async fn get_recent_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let rows = sqlx::query(
            r#"SELECT id, title, artist_id, year, cover_art, 
                (SELECT name FROM artists WHERE id = albums.artist_id) as artist_name,
                (SELECT COUNT(*) FROM tracks WHERE album_id = albums.id) as track_count
            FROM albums 
            ORDER BY created_at DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(map_row_to_album).collect())
    }

    async fn get_random_albums(&self, limit: u32) -> Result<Vec<Album>, String> {
        let rows = sqlx::query(
            r#"SELECT id, title, artist_id, year, cover_art, 
                (SELECT name FROM artists WHERE id = albums.artist_id) as artist_name,
                (SELECT COUNT(*) FROM tracks WHERE album_id = albums.id) as track_count
            FROM albums 
            ORDER BY RANDOM() LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(map_row_to_album).collect())
    }

    async fn get_most_played_tracks(&self, limit: u32) -> Result<Vec<Track>, String> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as artist_name, al.title as album_title
            FROM tracks t
            LEFT JOIN artists a ON t.artist_id = a.id
            LEFT JOIN albums al ON t.album_id = al.id
            WHERE t.play_count > 0
            ORDER BY t.play_count DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| map_row_to_track(r, Some(self.id.clone())))
            .collect())
    }

    async fn get_library_stats(&self) -> Result<crate::models::entities::LibraryStats, String> {
        let album_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM albums")
            .fetch_one(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        let track_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tracks")
            .fetch_one(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        let artist_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM artists")
            .fetch_one(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        let total_duration: i64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(duration_sec), 0) FROM tracks")
                .fetch_one(&self.db)
                .await
                .map_err(|e| e.to_string())?;

        let average_bitrate: i64 = sqlx::query_scalar(
            "SELECT CAST(COALESCE(AVG(bitrate), 0) AS INTEGER) FROM tracks WHERE bitrate IS NOT NULL",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(crate::models::entities::LibraryStats {
            album_count: album_count as u32,
            track_count: track_count as u32,
            artist_count: artist_count as u32,
            total_duration: total_duration.try_into().unwrap_or(u32::MAX),
            average_bitrate: average_bitrate.try_into().unwrap_or(u32::MAX),
        })
    }

    async fn get_genres(&self) -> Result<Vec<Genre>, String> {
        let rows = sqlx::query(
            r#"SELECT genre as name, COUNT(*) as track_count 
            FROM tracks 
            WHERE genre IS NOT NULL AND genre != '' 
            GROUP BY genre 
            ORDER BY track_count DESC"#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|row| Genre {
                name: row.get("name"),
                track_count: row.get::<i64, _>("track_count") as u32,
            })
            .collect())
    }

    async fn get_genre_tracks(&self, genre: &str) -> Result<Vec<Track>, String> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as artist_name, al.title as album_title
            FROM tracks t
            LEFT JOIN artists a ON t.artist_id = a.id
            LEFT JOIN albums al ON t.album_id = al.id
            WHERE t.genre = ?
            ORDER BY t.play_count DESC"#,
        )
        .bind(genre)
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| map_row_to_track(r, Some(self.id.clone())))
            .collect())
    }

    async fn get_favorites(&self) -> Result<Vec<Track>, String> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as artist_name, al.title as album_title
            FROM tracks t
            LEFT JOIN artists a ON t.artist_id = a.id
            LEFT JOIN albums al ON t.album_id = al.id
            WHERE t.liked = 1
            ORDER BY t.created_at DESC"#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| map_row_to_track(r, Some(self.id.clone())))
            .collect())
    }

    async fn search(&self, query: &str) -> Result<UnifiedSearchResult, String> {
        let pattern = format!("%{}%", query);

        let tracks_future = sqlx::query(
            r#"SELECT DISTINCT t.*, a.name as artist_name, al.title as album_title 
               FROM tracks t 
               LEFT JOIN artists a ON t.artist_id = a.id
               LEFT JOIN albums al ON t.album_id = al.id
               WHERE t.title LIKE ? OR a.name LIKE ? LIMIT 20"#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.db);

        let albums_future = sqlx::query(
            r#"SELECT 
                al.id, al.title, al.artist_id, al.year, al.cover_art, 
                ar.name as artist_name,
                (SELECT COUNT(*) FROM tracks WHERE album_id = al.id) as track_count
            FROM albums al
            LEFT JOIN artists ar ON al.artist_id = ar.id
            WHERE al.title LIKE ? OR ar.name LIKE ?
            LIMIT 20"#,
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.db);

        let artists_future = sqlx::query(r#"SELECT * FROM artists WHERE name LIKE ? LIMIT 20"#)
            .bind(&pattern)
            .fetch_all(&self.db);

        let (track_rows, album_rows, artist_rows) =
            tokio::try_join!(tracks_future, albums_future, artists_future)
                .map_err(|e| e.to_string())?;

        Ok(UnifiedSearchResult {
            tracks: track_rows
                .into_iter()
                .map(|r| map_row_to_track(r, Some(self.id.clone())))
                .collect(),
            albums: album_rows.into_iter().map(map_row_to_album).collect(),
            artists: artist_rows
                .into_iter()
                .map(|row| Artist {
                    id: row.get("id"),
                    name: row.get("name"),
                    bio: row.try_get("bio").unwrap_or_default(),
                    image_url: row.try_get("image_url").unwrap_or_default(),
                })
                .collect(),
        })
    }

    async fn get_artist(&self, id: &str) -> Result<Artist, String> {
        let row = sqlx::query("SELECT * FROM artists WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Artist not found".to_string())?;
        Ok(Artist {
            id: row.get("id"),
            name: row.get("name"),
            bio: row.try_get("bio").unwrap_or_default(),
            image_url: row.try_get("image_url").unwrap_or_default(),
        })
    }
    async fn get_track(&self, track_id: &str) -> Result<Track, String> {
        let row = sqlx::query(r#"SELECT t.*, a.name as artist_name, al.title as album_title FROM tracks t LEFT JOIN artists a ON t.artist_id = a.id LEFT JOIN albums al ON t.album_id = al.id WHERE t.id = ?"#).bind(track_id).fetch_optional(&self.db).await.map_err(|e| e.to_string())?.ok_or("Track not found".to_string())?;
        Ok(map_row_to_track(row, Some(self.id.clone())))
    }
    async fn get_album(&self, album_id: &str) -> Result<Album, String> {
        let row = sqlx::query(r#"SELECT id, title, artist_id, year, cover_art, (SELECT name FROM artists WHERE id = albums.artist_id) as artist_name, (SELECT COUNT(*) FROM tracks WHERE album_id = albums.id) as track_count FROM albums WHERE id = ?"#).bind(album_id).fetch_optional(&self.db).await.map_err(|e| e.to_string())?.ok_or("Album not found".to_string())?;
        Ok(map_row_to_album(row))
    }
    async fn set_track_liked(&self, track_id: &str, liked: bool) -> Result<(), String> {
        sqlx::query("UPDATE tracks SET liked = ? WHERE id = ?")
            .bind(liked)
            .bind(track_id)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        let rows = sqlx::query(r#"SELECT p.*, (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as track_count FROM playlists p ORDER BY created_at DESC"#).fetch_all(&self.db).await.map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|row| Playlist {
                id: row.get("id"),
                name: row.get("name"),
                owner: row.try_get("owner").unwrap_or_default(),
                track_count: row.try_get::<i64, _>("track_count").unwrap_or(0) as u32,
                cover_art: row.try_get("cover_art").ok(),
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .collect())
    }
    async fn create_playlist(&self, name: &str) -> Result<Playlist, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query("INSERT INTO playlists (id, name, created_at) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(name)
            .bind(now)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Playlist {
            id,
            name: name.to_string(),
            owner: "local".to_string(),
            track_count: 0,
            cover_art: None,
            created_at: now,
        })
    }
    async fn delete_playlist(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn add_to_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        let row = sqlx::query(
            "SELECT MAX(position) as max_pos FROM playlist_tracks WHERE playlist_id = ?",
        )
        .bind(playlist_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        let position: i32 = row.try_get("max_pos").unwrap_or(0) + 1;
        sqlx::query("INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)").bind(playlist_id).bind(track_id).bind(position).execute(&self.db).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn remove_from_playlist(&self, playlist_id: &str, track_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
            .bind(playlist_id)
            .bind(track_id)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn resolve_stream(&self, track_id: &str) -> Result<AudioStream, String> {
        let row = sqlx::query("SELECT path FROM tracks WHERE id = ?")
            .bind(track_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Track not found".to_string())?;
        Ok(AudioStream::Url(row.get("path")))
    }
    async fn scan(&self) -> Result<(), String> {
        let rows = sqlx::query("SELECT path FROM library_roots")
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        let existing_tracks_rows = sqlx::query("SELECT path, mtime FROM tracks")
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        let mut existing_map: HashMap<PathBuf, i64> =
            HashMap::with_capacity(existing_tracks_rows.len());
        for row in existing_tracks_rows {
            let p: String = row.get("path");
            let m: i64 = row.try_get("mtime").unwrap_or(0);
            existing_map.insert(PathBuf::from(p), m);
        }
        let existing_map_arc = Arc::new(existing_map);
        sqlx::query("DELETE FROM scan_found")
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        for row in rows {
            let path: String = row.get("path");
            if let Err(e) = self.scan_path(&path, existing_map_arc.clone()).await {
                log::error!("Scan failed for root {}: {}", path, e);
                return Err(format!("Scan failed for root {}: {}", path, e));
            }
        }
        let _ = sqlx::query("DELETE FROM tracks WHERE path NOT IN (SELECT path FROM scan_found)")
            .execute(&self.db)
            .await;
        let _ = sqlx::query("PRAGMA optimize").execute(&self.db).await;

        if let Some(lastfm_config) = &self.config.lastfm {
            if lastfm_config.enabled {
                log::info!("Last.fm enabled. Fetching artist metadata...");
                let client = LastFmClient::new(
                    lastfm_config.api_key.clone(),
                    lastfm_config.api_secret.clone(),
                    lastfm_config.username.clone(),
                );

                let artists: Vec<(String, String)> = sqlx::query_as(
                    "SELECT id, name FROM artists WHERE bio IS NULL OR image_url IS NULL",
                )
                .fetch_all(&self.db)
                .await
                .map_err(|e| e.to_string())?;

                let client = Arc::new(client);
                let db_pool = self.db.clone();

                futures::stream::iter(artists)
                    .map(|(id, name)| {
                        let client = client.clone();
                        let db_pool = db_pool.clone();
                        async move {
                            if name == "Unknown Artist" {
                                return;
                            }

                            let mut attempts = 0;
                            loop {
                                match client.get_artist_info(&name).await {
                                    Ok(info) => {
                                        let mut bio = None;
                                        let mut image_url = None;

                                        if let Some(b) = info.bio {
                                            bio = Some(b.content);
                                        }

                                        if let Some(images) = info.image {
                                            if let Some(img) = images
                                                .iter()
                                                .find(|i| i.size == "mega")
                                                .or(images.last())
                                            {
                                                if !img.url.is_empty() {
                                                    image_url = Some(img.url.clone());
                                                }
                                            }
                                        }

                                        if bio.is_some() || image_url.is_some() {
                                            let _ = sqlx::query("UPDATE artists SET bio = COALESCE(?, bio), image_url = COALESCE(?, image_url) WHERE id = ?")
                                                .bind(bio)
                                                .bind(image_url)
                                                .bind(&id)
                                                .execute(&db_pool)
                                                .await;
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        let err_str = e.to_string();
                                        if err_str.contains("429") {
                                            log::warn!("Last.fm Rate Limit (429) for {}. Waiting...", name);
                                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                                            continue;
                                        }

                                        attempts += 1;
                                        if attempts >= 3 {
                                            log::warn!(
                                                "Failed to fetch Last.fm info for {} after 3 attempts: {}",
                                                name,
                                                e
                                            );
                                            break;
                                        }
                                        tokio::time::sleep(tokio::time::Duration::from_millis(500 * attempts as u64)).await;
                                    }
                                }
                            }
                        }
                    })
                    .buffer_unordered(20)
                    .collect::<Vec<()>>()
                    .await;
            }
        }

        Ok(())
    }

    async fn add_root(&self, path: &str) -> Result<(), String> {
        sqlx::query("INSERT OR IGNORE INTO library_roots (path) VALUES (?)")
            .bind(path)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, String> {
        let rows = sqlx::query(r#"SELECT t.*, a.name as artist_name, al.title as album_title FROM playlist_tracks pt JOIN tracks t ON pt.track_id = t.id LEFT JOIN artists a ON t.artist_id = a.id LEFT JOIN albums al ON t.album_id = al.id WHERE pt.playlist_id = ? ORDER BY pt.position ASC"#).bind(playlist_id).fetch_all(&self.db).await.map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|r| map_row_to_track(r, Some(self.id.clone())))
            .collect())
    }
}

fn map_row_to_album(row: sqlx::sqlite::SqliteRow) -> Album {
    Album {
        id: row.get("id"),
        title: row.get("title"),
        artist_id: row.try_get("artist_id").unwrap_or_default(),
        artist_name: row.try_get("artist_name").unwrap_or_default(),
        cover_art: row.try_get("cover_art").unwrap_or_default(),
        year: row
            .try_get::<Option<i64>, _>("year")
            .unwrap_or_default()
            .map(|y| y as u16),
        track_count: row
            .try_get::<Option<i64>, _>("track_count")
            .unwrap_or_default()
            .map(|c| c as u32),
    }
}

fn map_row_to_track(row: sqlx::sqlite::SqliteRow, provider_id: Option<String>) -> Track {
    Track {
        id: row.get("id"),
        provider_id,
        title: row.get("title"),
        artist_id: row.try_get("artist_id").unwrap_or_default(),
        artist_name: row.try_get("artist_name").unwrap_or_default(),
        album_id: row.try_get("album_id").unwrap_or_default(),
        album_title: row.try_get("album_title").unwrap_or_default(),
        duration_sec: row.try_get("duration_sec").unwrap_or_default(),
        track_number: row.try_get("track_number").ok(),
        disc_number: row.try_get("disc_number").ok(),
        year: row.try_get("year").ok(),
        genre: row.try_get("genre").ok(),
        bitrate: row.try_get("bitrate").ok(),
        play_count: row.try_get("play_count").unwrap_or(0),
        liked: row.try_get("liked").unwrap_or(false),
    }
}
