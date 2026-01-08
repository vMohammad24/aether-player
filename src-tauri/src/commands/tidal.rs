use crate::providers::tidal::{DeviceAuthPending, TidalProvider};

#[tauri::command]
#[specta::specta]
pub async fn start_tidal_login() -> Result<DeviceAuthPending, String> {
    TidalProvider::start_device_auth()
        .await
        .map_err(|e| format!("Failed to start Tidal login: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn poll_tidal_login(config: DeviceAuthPending) -> Result<(), String> {
    TidalProvider::poll_device_token(&config)
        .await
        .map_err(|e| format!("Failed to complete Tidal login: {}", e))?;
    Ok(())
}
