use crate::models::config::SourceConfig;
use crate::providers::tidal::{DeviceAuthPending, TidalProvider};
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;
#[tauri::command]
#[specta::specta]
pub async fn start_tidal_login() -> Result<DeviceAuthPending, String> {
    TidalProvider::start_device_auth()
        .await
        .map_err(|e| format!("Failed to start Tidal login: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn poll_tidal_login(
    state: State<'_, AppState>,
    app: AppHandle,
    authConfig: DeviceAuthPending,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;
    let mut config: crate::models::AppConfig = if let Some(val) = store.get("appConfig") {
        serde_json::from_value(val).map_err(|e| format!("Config error: {}", e))?
    } else {
        crate::models::AppConfig::default()
    };
    let credentials = TidalProvider::poll_device_token(&authConfig)
        .await
        .map_err(|e| format!("Failed to complete Tidal login: {}", e))?;

    let source_id = "tidal".to_string();
    let source_name = "Tidal".to_string();

    let new_source = SourceConfig::Tidal {
        id: source_id.clone(),
        name: source_name.clone(),
        access_token: credentials.access_token.clone(),
        refresh_token: credentials.refresh_token.clone(),
        expires_at: credentials.expires_at,
        user_id: credentials.user_id.clone(),
        country_code: credentials.country_code.clone(),
        scopes: credentials.scopes.clone(),
        enabled: true,
    };

    config.sources.retain(|s| !matches!(s, SourceConfig::Tidal { .. }));
    config.sources.push(new_source);

    let val = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    store.set("appConfig", val);
    store.save().map_err(|e| e.to_string())?;

    let provider = TidalProvider::new(source_id, source_name, credentials)
        .await
        .map_err(|e| e.to_string())?;
    state.queue.add_provider(Arc::new(provider)).await;

    Ok(())
}
