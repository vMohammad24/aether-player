use crate::models::AppConfig;
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

#[tauri::command]
#[specta::specta]
pub fn get_default_config() -> AppConfig {
    AppConfig::default()
}

#[tauri::command]
#[specta::specta]
pub fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;

    if let Some(val) = store.get("appConfig") {
        serde_json::from_value(val).map_err(|e| format!("Failed to parse config: {}", e))
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
#[specta::specta]
pub async fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;

    let val = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    store.set("appConfig", val);
    store.save().map_err(|e| e.to_string())?;

    if let Some(state) = app.try_state::<AppState>() {
        if let Some(discord_config) = &config.discord_rpc {
            let mut discord = state.discord.lock().await;
            discord.update_config(discord_config.clone());
        }
    }

    Ok(())
}