use crate::models::player::AudioDevice;
use crate::models::{player::PlayerState, player::RepeatMode};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn play_track(state: State<'_, AppState>, track_id: String) -> Result<(), String> {
    let track = state
        .queue
        .get_track(&track_id)
        .await
        .ok_or("Track not found in any provider".to_string())?;
    state.queue.play_now(track).await
}

#[tauri::command]
#[specta::specta]
pub async fn play(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.player.play().await
}

#[tauri::command]
#[specta::specta]
pub async fn pause(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.player.pause().await
}

#[tauri::command]
#[specta::specta]
pub async fn stop(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.player.stop().await
}

#[tauri::command]
#[specta::specta]
pub async fn next(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.next().await
}

#[tauri::command]
#[specta::specta]
pub async fn prev(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.prev().await
}

#[tauri::command]
#[specta::specta]
pub async fn seek(state: State<'_, AppState>, seconds: f64) -> Result<(), String> {
    state.queue.player.seek(seconds).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    state.queue.player.set_volume(volume).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_repeat(state: State<'_, AppState>, mode: RepeatMode) -> Result<(), String> {
    state.queue.set_repeat(mode).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_shuffle(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.queue.toggle_shuffle().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_player_state(state: State<'_, AppState>) -> Result<PlayerState, String> {
    Ok(state.queue.player.get_state().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_audio_devices(state: State<'_, AppState>) -> Result<Vec<AudioDevice>, String> {
    state.queue.player.get_audio_devices().await
}
#[tauri::command]
#[specta::specta]
pub async fn set_audio_device(
    state: State<'_, AppState>,
    device_id: Option<String>,
) -> Result<(), String> {
    state.queue.player.set_audio_device(device_id).await
}
