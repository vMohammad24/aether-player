use crate::models::config::SourceConfig;
use crate::models::entities::{Album, Artist, Playlist, Track, UnifiedSearchResult};
use crate::providers::local::LocalProvider;
use crate::state::AppState;
use crate::traits::LibraryProvider;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

#[tauri::command]
#[specta::specta]
pub async fn add_source(
    state: State<'_, AppState>,
    app: AppHandle,
    source: SourceConfig,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    let mut config: crate::models::AppConfig = if let Some(val) = store.get("appConfig") {
        serde_json::from_value(val).map_err(|e| format!("Config error: {}", e))?
    } else {
        crate::models::AppConfig::default()
    };

    config.sources.push(source.clone());
    let val = serde_json::to_value(config).map_err(|e| e.to_string())?;
    store.set("appConfig", val);
    store.save().map_err(|e| e.to_string())?;

    match source {
        SourceConfig::Local { id, path, .. } => {
            let app_data_dir = dirs::data_local_dir()
                .ok_or("failed to get local data dir")?
                .join(crate::APP_IDENTIFIER);
            let db_path = app_data_dir.join(format!("library_{}.db", id));

            let provider = LocalProvider::new(id.clone(), &db_path, &app_data_dir)
                .await
                .map_err(|e| e.to_string())?;

            provider.add_root(&path).await?;
            provider.scan().await?;

            state
                .queue
                .add_provider(std::sync::Arc::new(provider))
                .await;
        }
        _ => return Err("Provider type not implemented yet".to_string()),
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_source(
    state: State<'_, AppState>,
    app: AppHandle,
    source_id: String,
) -> Result<(), String> {
    state.queue.remove_provider(&source_id).await;

    let store = app.store("config.json").map_err(|e| e.to_string())?;
    let mut config: crate::models::AppConfig = if let Some(val) = store.get("appConfig") {
        serde_json::from_value(val).map_err(|e| format!("Config error: {}", e))?
    } else {
        crate::models::AppConfig::default()
    };

    if let Some(source) = config.sources.iter().find(|s| match s {
        SourceConfig::Local { id, .. } => id == &source_id,
        _ => false,
    }) {
        if let SourceConfig::Local { id, .. } = source {
            if let Some(app_data_dir) = dirs::data_local_dir() {
                let data_dir = app_data_dir.join(crate::APP_IDENTIFIER);
                let db_path = data_dir.join(format!("library_{}.db", id));
                if db_path.exists() {
                    let _ = std::fs::remove_file(db_path);
                }
            }
        }
    }

    config.sources.retain(|s| match s {
        SourceConfig::Local { id, .. } => id != &source_id,
        SourceConfig::Subsonic { id, .. } => id != &source_id,
        SourceConfig::Tidal { id, .. } => id != &source_id,
    });

    let val = serde_json::to_value(config).map_err(|e| e.to_string())?;
    store.set("appConfig", val);
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn scan_library(state: State<'_, AppState>) -> Result<(), String> {
    let providers = state.queue.get_providers().await;
    for provider in providers.values() {
        let _ = provider.scan().await;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn add_library_root(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let providers = state.queue.get_providers().await;
    let mut success = false;
    for provider in providers.values() {
        if provider.add_root(&path).await.is_ok() {
            success = true;
        }
    }
    if success {
        Ok(())
    } else {
        Err("No provider supported adding root or all failed".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_playlists(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<Vec<Playlist>, String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.get_playlists().await
}

#[tauri::command]
#[specta::specta]
pub async fn create_playlist(
    state: State<'_, AppState>,
    provider_id: String,
    name: String,
) -> Result<Playlist, String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.create_playlist(&name).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_playlist(
    state: State<'_, AppState>,
    provider_id: String,
    playlist_id: String,
) -> Result<(), String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.delete_playlist(&playlist_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_to_playlist(
    state: State<'_, AppState>,
    provider_id: String,
    playlist_id: String,
    track_id: String,
) -> Result<(), String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.add_to_playlist(&playlist_id, &track_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_from_playlist(
    state: State<'_, AppState>,
    provider_id: String,
    playlist_id: String,
    track_id: String,
) -> Result<(), String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.remove_from_playlist(&playlist_id, &track_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_playlist_tracks(
    state: State<'_, AppState>,
    provider_id: String,
    playlist_id: String,
) -> Result<Vec<Track>, String> {
    let provider = state
        .queue
        .get_provider(&provider_id)
        .await
        .ok_or("Provider not found".to_string())?;
    provider.get_playlist_tracks(&playlist_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_recent_albums(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<Album>, String> {
    let providers = state.queue.get_providers().await;
    let mut all_albums = Vec::new();
    for provider in providers.values() {
        if let Ok(mut albums) = provider.get_recent_albums(limit).await {
            all_albums.append(&mut albums);
        }
    }
    Ok(all_albums)
}

#[tauri::command]
#[specta::specta]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    let providers = state.queue.get_providers().await;
    let mut all_tracks = Vec::new();
    for provider in providers.values() {
        if let Ok(mut tracks) = provider.get_favorites().await {
            all_tracks.append(&mut tracks);
        }
    }
    Ok(all_tracks)
}

#[tauri::command]
#[specta::specta]
pub async fn set_favorite(
    state: State<'_, AppState>,
    track_id: String,
    liked: bool,
) -> Result<(), String> {
    let providers = state.queue.get_providers().await;
    for provider in providers.values() {
        let _ = provider.set_track_liked(&track_id, liked).await;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
) -> Result<UnifiedSearchResult, String> {
    let providers = state.queue.get_providers().await;
    let mut result = UnifiedSearchResult::default();

    for provider in providers.values() {
        if let Ok(res) = provider.search(&query).await {
            result.tracks.extend(res.tracks);
            result.albums.extend(res.albums);
            result.artists.extend(res.artists);
        }
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_artist(state: State<'_, AppState>, artist_id: String) -> Result<Artist, String> {
    let providers = state.queue.get_providers().await;
    for provider in providers.values() {
        if let Ok(artist) = provider.get_artist(&artist_id).await {
            return Ok(artist);
        }
    }
    Err("Artist not found".to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_artist_albums(
    state: State<'_, AppState>,
    artist_id: String,
) -> Result<Vec<Album>, String> {
    let providers = state.queue.get_providers().await;
    let mut all_albums = Vec::new();
    for provider in providers.values() {
        if let Ok(mut albums) = provider.get_artist_albums(&artist_id).await {
            all_albums.append(&mut albums);
        }
    }
    Ok(all_albums)
}

#[tauri::command]
#[specta::specta]
pub async fn get_album_tracks(
    state: State<'_, AppState>,
    album_id: String,
) -> Result<Vec<Track>, String> {
    let providers = state.queue.get_providers().await;
    let mut all_tracks = Vec::new();
    for provider in providers.values() {
        if let Ok(mut tracks) = provider.get_album_tracks(&album_id).await {
            all_tracks.append(&mut tracks);
        }
    }
    Ok(all_tracks)
}
