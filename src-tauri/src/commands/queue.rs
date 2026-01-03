use crate::models::player::Queue;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn get_queue(state: State<'_, AppState>) -> Result<Queue, String> {
    Ok(state.queue.get_queue().await)
}

#[tauri::command]
#[specta::specta]
pub async fn add_to_queue(
    state: State<'_, AppState>,
    track_id: String,
) -> Result<(), String> {
    let track = state
        .queue
        .get_track(&track_id)
        .await
        .ok_or("Track not found in any provider".to_string())?;
    state.queue.add_track(track).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn add_next(
    state: State<'_, AppState>,
    track_id: String,
) -> Result<(), String> {
    let track = state
        .queue
        .get_track(&track_id)
        .await
        .ok_or("Track not found in any provider".to_string())?;
    state.queue.add_next(track).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_from_queue(state: State<'_, AppState>, index: u32) -> Result<(), String> {
    state.queue.remove(index as usize).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn clear_queue(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.clear().await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn play_from_queue(state: State<'_, AppState>, index: u32) -> Result<(), String> {
    state.queue.play_index(index as usize).await
}
