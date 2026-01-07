use crate::models::config::LastFmSessionConfig;
use crate::models::AppConfig;
use crate::state::AppState;
use crate::util::lastfm::LastFmClient;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(serde::Serialize, specta::Type)]
pub struct LastFmAuthUrl {
    pub url: String,
    pub token: String,
}

#[tauri::command]
#[specta::specta]
pub async fn login_lastfm(_app: AppHandle) -> Result<LastFmAuthUrl, String> {
    let client = LastFmClient::new(None, None);

    let token = client.get_token().await.map_err(|e| e.to_string())?;
    let url = format!(
        "http://www.last.fm/api/auth/?api_key={}&token={}",
        env!("LASTFM_API_KEY"),
        token
    );

    Ok(LastFmAuthUrl { url, token })
}

#[tauri::command]
#[specta::specta]
pub async fn finish_lastfm_login(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    token: String,
) -> Result<(), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;

    let mut config: AppConfig = if let Some(val) = store.get("appConfig") {
        serde_json::from_value(val).map_err(|e| e.to_string())?
    } else {
        return Err("Configuration not found".to_string());
    };

    let client = LastFmClient::new(None, None);

    let session = client
        .get_session(&token)
        .await
        .map_err(|e| e.to_string())?;

    config.lastfm_session = Some(LastFmSessionConfig {
        username: session.name.clone(),
        session_key: session.key.clone(),
        enabled: true,
    });

    let val = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    store.set("appConfig", val);
    store.save().map_err(|e| e.to_string())?;

    let mut state_lfm = state.lastfm.lock().await;
    let new_client = LastFmClient::new(Some(session.name), Some(session.key));
    *state_lfm = Some(new_client);

    Ok(())
}
