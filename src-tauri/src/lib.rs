// NOTE: To the unfortunate soul tasked with maintaining this code; heed my words.
// Ruin has befallen this once-proud repository.
// You remember its prime, a bastion of clean logic and elegant design,
// deployed with pride upon the finest of web servers.
// Now, behold its descent into chaos. It has become The Pit.
//
// I must confess the sins of my hand:
// Mistakes were sown, and the festering rot of technical debt has spread unchecked.
// To implement even the simplest feature now is to face a Sisyphean torment-duplicated, twisted, sixfold over.
// My efforts to cleanse this blight were stifled, for I was bound by the cruelest of masters: the deadline.
// In my haste to deliver, quality was cast aside, sacrificed upon the altar of urgency.
//
// I beg you,
// Turn back the clock. Revert the commits. Rebuild it, lest the bottomless pit claim us all.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod commands;
mod models;
mod players;
mod providers;
mod queue;
mod state;
mod traits;
pub mod util;

use crate::models::{config::SourceConfig, AppConfig, AudioBackend};
use crate::players::mpv::MpvPlayer;
use crate::providers::local::LocalProvider;
use crate::providers::subsonic::SubsonicProvider;
use crate::queue::QueueManager;
use crate::state::AppState;
use crate::traits::{AudioEngine, LibraryProvider};
use std::collections::HashMap;
use tauri::Manager;

fn create_audio_engine(config: &AppConfig) -> anyhow::Result<Box<dyn AudioEngine>> {
    match &config.audio_engine {
        AudioBackend::Mpv(mpv_opts) => Ok(Box::new(MpvPlayer::new(mpv_opts.clone())?)),
    }
}

pub const APP_IDENTIFIER: &str = "dev.vmohammad.aether";

pub async fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::player::play_track,
            commands::player::play,
            commands::player::pause,
            commands::player::stop,
            commands::player::next,
            commands::player::prev,
            commands::player::seek,
            commands::player::set_volume,
            commands::player::set_repeat,
            commands::player::toggle_shuffle,
            commands::player::get_player_state,
            commands::player::get_audio_devices,
            commands::player::set_audio_device,
            commands::player::toggle_exclusive_mode,
            commands::queue::get_queue,
            commands::queue::add_to_queue,
            commands::queue::add_to_queue_multiple,
            commands::queue::add_next,
            commands::queue::remove_from_queue,
            commands::queue::clear_queue,
            commands::queue::play_from_queue,
            commands::library::scan_libraries,
            commands::library::scan_library,
            commands::library::add_library_root,
            commands::library::get_playlists,
            commands::library::create_playlist,
            commands::library::delete_playlist,
            commands::library::add_to_playlist,
            commands::library::remove_from_playlist,
            commands::library::get_playlist_tracks,
            commands::library::get_recent_albums,
            commands::library::get_random_albums,
            commands::library::get_most_played_tracks,
            commands::library::get_genres,
            commands::library::get_genre_tracks,
            commands::library::get_library_stats,
            commands::library::get_favorites,
            commands::library::search,
            commands::library::get_artist,
            commands::library::get_album,
            commands::library::get_artist_albums,
            commands::library::get_album_tracks,
            commands::library::set_favorite,
            commands::library::add_source,
            commands::library::delete_source,
            commands::library::toggle_source,
            commands::config::get_default_config,
            commands::config::get_app_config,
            commands::config::save_app_config,
            commands::lastfm::login_lastfm,
            commands::lastfm::finish_lastfm_login,
        ])
        .events(tauri_specta::collect_events![
            crate::models::entities::PlayerEvent
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    use std::sync::Arc;
    let providers: HashMap<String, Arc<dyn LibraryProvider>> = HashMap::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            #[cfg(desktop)]
            {
                let _ = app
                    .get_webview_window("main")
                    .expect("no main window")
                    .set_focus();
            }
        }))
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let config = AppConfig::load(app.handle()).unwrap_or_else(|e| {
                log::error!("Failed to load appConfig: {}. Using default.", e);
                AppConfig::default()
            });

            let player = create_audio_engine(&config)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let state_path = app_data_dir.join("playback_state.json");

            let queue = QueueManager::new(player, providers, state_path);

            let mut lastfm_client = None;
            if let Some(lfm_config) = &config.lastfm_session {
                if lfm_config.enabled {
                    let client = crate::util::lastfm::LastFmClient::new(
                        Some(lfm_config.username.clone()),
                        Some(lfm_config.session_key.clone()),
                    );
                    lastfm_client = Some(client);
                }
            }

            let discord_config = config.discord_rpc.clone().unwrap_or_default();
            let discord_rpc = crate::util::discord::DiscordRpc::new(discord_config);

            app.manage(AppState::new(
                queue.clone(),
                lastfm_client.clone(),
                discord_rpc,
            ));

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                for source in &config.sources {
                    match source {
                        SourceConfig::Local {
                            id, path, enabled, ..
                        } => {
                            if !enabled {
                                continue;
                            }
                            if let Some(app_data_dir) = dirs::data_local_dir() {
                                let data_dir = app_data_dir.join(crate::APP_IDENTIFIER);
                                let db_path = data_dir.join(format!("library_{}.db", id));

                                if let Ok(provider) = LocalProvider::new(
                                    id.clone(),
                                    &db_path,
                                    &data_dir,
                                    config.clone(),
                                )
                                .await
                                {
                                    let _ = provider.add_root(&path).await;

                                    queue.add_provider(Arc::new(provider)).await;
                                }
                            }
                        }
                        SourceConfig::Subsonic {
                            id,
                            name,
                            url,
                            username,
                            token,
                            salt,
                            enabled,
                        } => {
                            if !enabled {
                                continue;
                            }
                            if let Ok(provider) = SubsonicProvider::new(
                                id.clone(),
                                name.clone(),
                                url.clone(),
                                username.clone(),
                                token.clone(),
                                salt.clone(),
                            ) {
                                queue.add_provider(Arc::new(provider)).await;
                            }
                        }
                    }
                }

                queue.load_state().await;

                let state = handle.state::<AppState>();

                if let Some(target_device) = &config.audio_output_device {
                    match &config.audio_engine {
                        AudioBackend::Mpv(mpv_opts) => {
                            if mpv_opts.audio_device.is_none() {
                                let player = &state.queue.player;
                                match player.get_audio_devices().await {
                                    Ok(devices) => {
                                        if devices.iter().any(|d| d.id == *target_device) {
                                            if let Err(e) = player
                                                .set_audio_device(Some(target_device.clone()))
                                                .await
                                            {
                                                log::error!(
                                                    "Failed to set audio device to {}: {}",
                                                    target_device,
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => log::error!("Failed to get audio devices: {}", e),
                                }
                            }
                        }
                    }
                }
                crate::util::lastfm::start_scrobbling_service(
                    state.queue.clone(),
                    state.lastfm.clone(),
                );
                crate::util::discord::start_discord_rpc_service(
                    state.queue.clone(),
                    state.discord.clone(),
                    state.lastfm.clone(),
                );

                use tauri_specta::Event;
                let mut rx = queue.player.subscribe();

                while let Ok(event) = rx.recv().await {
                    let _ = event.emit(&handle);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
