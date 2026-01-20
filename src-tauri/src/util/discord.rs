use crate::models::config::DiscordRpcConfig;
use crate::models::entities::{PlayerEvent, Track};
use crate::queue::QueueManager;
use crate::util::lastfm::LastFmClient;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

const DISCORD_APP_ID: &str = "1458263853203853477";
const DEFAULT_LARGE_IMAGE: &str = "icon";
const DEFAULT_ALBUM_TEXT: &str = "Unknown album";
const DEFAULT_ARTIST_TEXT: &str = "Unknown artist";
const PAUSE_IMAGE: &str = "pause";

pub struct DiscordRpc {
    client: Option<DiscordIpcClient>,
    config: DiscordRpcConfig,
}

impl DiscordRpc {
    pub fn new(config: DiscordRpcConfig) -> Self {
        Self {
            client: None,
            config,
        }
    }

    pub fn update_config(&mut self, config: DiscordRpcConfig) {
        let was_enabled = self.config.enabled;
        self.config = config;

        if self.config.enabled && !was_enabled {
            self.connect();
        } else if !self.config.enabled && was_enabled {
            self.close();
        }
    }

    fn connect(&mut self) {
        if !self.config.enabled || self.client.is_some() {
            return;
        }

        let mut client = DiscordIpcClient::new(DISCORD_APP_ID);
        if let Err(e) = client.connect() {
            log::warn!("Failed to connect to Discord IPC: {}", e);
        } else {
            log::info!("Connected to Discord RPC");
            self.client = Some(client);
        }
    }

    fn close(&mut self) {
        if let Some(mut client) = self.client.take() {
            let _ = client.close();
        }
    }

    fn ensure_connected(&mut self) -> bool {
        if !self.config.enabled {
            return false;
        }

        if self.client.is_none() {
            self.connect();
        }

        self.client.is_some()
    }

    pub fn update_presence(
        &mut self,
        track: &Track,
        duration: f64,
        position: f64,
        is_playing: bool,
        large_image_url: Option<String>,
        artist_image_url: Option<String>,
    ) {
        if !self.ensure_connected() {
            return;
        }

        let details = if self.config.show_details {
            format_track_string(&self.config.details_format, track)
        } else {
            String::new()
        };

        let state = if self.config.show_state {
            format_track_string(&self.config.state_format, track)
        } else {
            String::new()
        };

        let show_artist_icon = self.config.show_artist_icon;
        let show_time = self.config.show_time;
        let activity_on_pause = self.config.activity_on_pause;

        let album_text = if track.album_title.is_empty() {
            DEFAULT_ALBUM_TEXT.to_string()
        } else {
            track.album_title.clone()
        };

        let artist_text = if track.artist_name.is_empty() {
            DEFAULT_ARTIST_TEXT.to_string()
        } else {
            track.artist_name.clone()
        };

        if let Some(client) = &mut self.client {
            let mut activity = activity::Activity::new();
            activity = activity.activity_type(activity::ActivityType::Listening);

            if !details.is_empty() {
                activity = activity.details(&details);
            }

            if !state.is_empty() {
                activity = activity.state(&state);
            }

            let mut assets = activity::Assets::new();
            if let Some(url) = &large_image_url {
                assets = assets.large_image(url);
            } else {
                assets = assets.large_image(DEFAULT_LARGE_IMAGE);
            }
            assets = assets.large_text(&album_text);

            if is_playing {
                if show_artist_icon {
                    if let Some(url) = &artist_image_url {
                        assets = assets.small_image(url);
                        assets = assets.small_text(&artist_text);
                    }
                }

                if show_time {
                    if let Some(now) = now_unix_seconds() {
                        let clamped_duration = duration.max(0.0) as i64;
                        let clamped_position = position.max(0.0).min(duration.max(0.0)) as i64;
                        let start = now - clamped_position;
                        let end = start + clamped_duration;

                        if end > start {
                            activity = activity
                                .timestamps(activity::Timestamps::new().start(start).end(end));
                        }
                    }
                }
            } else {
                if activity_on_pause {
                    assets = assets.small_image(PAUSE_IMAGE);
                    assets = assets.small_text("Paused");
                }
            }

            activity = activity.assets(assets);

            if let Err(e) = client.set_activity(activity) {
                log::warn!("Failed to set Discord activity: {}", e);
            }
        }
    }

    pub fn clear(&mut self) {
        if let Some(client) = &mut self.client {
            let _ = client.clear_activity();
        }
    }
}

fn format_track_string(format: &str, track: &Track) -> String {
    format
        .replace("{track}", &track.title)
        .replace("{artist}", &track.artist_name)
        .replace("{album}", &track.album_title)
}

fn now_unix_seconds() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs() as i64)
}

pub fn start_discord_rpc_service(
    queue: Arc<QueueManager>,
    discord: Arc<Mutex<DiscordRpc>>,
    lastfm: Arc<Mutex<Option<LastFmClient>>>,
) {
    tauri::async_runtime::spawn(async move {
        let mut rx = queue.player.subscribe();
        let mut last_track_id: Option<String> = None;
        let mut cached_large_image: Option<String> = None;
        let mut cached_artist_image: Option<String> = None;

        while let Ok(event) = rx.recv().await {
            match event {
                PlayerEvent::Playing | PlayerEvent::Paused => {}
                _ => continue,
            }

            if let Some(track) = queue.current_track().await {
                let track_changed = last_track_id.as_deref() != Some(&track.id);

                if track_changed {
                    cached_large_image = None;
                    cached_artist_image = None;

                    if let Some(pid) = &track.provider_id {
                        if let Some(provider) = queue.get_provider(pid).await {
                            if let Ok(album) = provider.get_album(&track.album_id).await {
                                if let Some(art) = album.cover_art {
                                    if art.starts_with("http") && !art.contains("getCoverArt") {
                                        cached_large_image = Some(art);
                                    }
                                }
                            }
                        }
                    }

                    let lfm_client = {
                        let guard = lastfm.lock().await;
                        guard.clone()
                    };

                    if let Some(client) = lfm_client {
                        if cached_large_image.is_none() {
                            if let Ok(info) = client
                                .get_track_info(&track.artist_name, &track.title)
                                .await
                            {
                                if let Some(images) = info.album.and_then(|a| a.image) {
                                    cached_large_image = images.last().map(|i| i.url.clone());
                                }
                            }
                        }

                        if let Ok(info) = client.get_artist_info(&track.artist_name).await {
                            if let Some(images) = info.image {
                                cached_artist_image = images.last().map(|i| i.url.clone());
                            }
                        }
                    }

                    last_track_id = Some(track.id.clone());
                }

                let mut discord = discord.lock().await;

                if let PlayerEvent::Paused = event {
                    discord.update_presence(
                        &track,
                        track.duration_sec as f64,
                        0.0,
                        false,
                        cached_large_image.clone(),
                        cached_artist_image.clone(),
                    );
                } else if let PlayerEvent::Playing = event {
                    discord.update_presence(
                        &track,
                        track.duration_sec as f64,
                        0.0,
                        true,
                        cached_large_image.clone(),
                        cached_artist_image.clone(),
                    );
                }
            } else {
                let mut discord = discord.lock().await;
                discord.clear();
            }
        }
    });
}
