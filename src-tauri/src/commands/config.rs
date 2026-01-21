use crate::models::AppConfig;
use crate::state::AppState;
use tauri::{AppHandle, Manager};

#[tauri::command]
#[specta::specta]
pub fn get_default_config() -> AppConfig {
    AppConfig::default()
}

#[tauri::command]
#[specta::specta]
pub fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    AppConfig::load(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save(&app)?;

    if let Some(state) = app.try_state::<AppState>() {
        if let Some(discord_config) = &config.discord_rpc {
            let mut discord = state.discord.lock().await;
            discord.update_config(discord_config.clone());
        }
    }

    Ok(())
}
