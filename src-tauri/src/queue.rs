use crate::models::{
    entities::{PlayerEvent, Track},
    player::{PersistedPlayer, PersistedQueue, PersistedState, Queue, RepeatMode},
};
use crate::traits::{AudioEngine, LibraryProvider};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct QueueManager {
    state: Mutex<QueueState>,
    pub player: Box<dyn AudioEngine>,
    providers: Arc<RwLock<HashMap<String, Arc<dyn LibraryProvider>>>>,
    state_path: PathBuf,
}

#[derive(Default)]
struct QueueState {
    tracks: Vec<Track>,
    current_index: Option<usize>,
    repeat_mode: RepeatMode,
    shuffle: bool,
    shuffled_indices: Vec<usize>,
}

impl QueueManager {
    pub fn new(
        player: Box<dyn AudioEngine>,
        providers: HashMap<String, Arc<dyn LibraryProvider>>,
        state_path: PathBuf,
    ) -> Arc<Self> {
        let providers = Arc::new(RwLock::new(providers));

        let initial_state = QueueState::default();

        let qm = Arc::new(Self {
            state: Mutex::new(initial_state),
            player,
            providers: providers.clone(),
            state_path,
        });

        let qm_clone = qm.clone();
        tokio::spawn(async move {
            let mut rx = qm_clone.player.subscribe();
            while let Ok(event) = rx.recv().await {
                match event {
                    PlayerEvent::Paused | PlayerEvent::Playing | PlayerEvent::Ended => {
                        let _ = qm_clone.save().await;
                    }
                    PlayerEvent::TimeUpdate(_) => {}
                    PlayerEvent::DurationChange(_) => {}
                    PlayerEvent::Error(_) => {}
                }

                if let PlayerEvent::Ended = event {
                    let _ = qm_clone.on_playback_ended().await;
                }
            }
        });

        qm
    }

    pub async fn load_state(&self) {
        if let Ok(content) = std::fs::read_to_string(&self.state_path) {
            if let Ok(persisted) = serde_json::from_str::<PersistedState>(&content) {
                let pq = persisted.queue;
                let mut tracks = Vec::new();
                for id in &pq.tracks {
                    if let Some(track) = self.get_track(id).await {
                        tracks.push(track);
                    }
                }

                let mut current_track_to_load = None;
                let mut state = self.state.lock().await;
                if tracks.len() == pq.tracks.len() {
                    state.tracks = tracks;
                    state.current_index = pq.current_index;
                    state.repeat_mode = pq.repeat_mode;
                    state.shuffle = pq.shuffle;
                    state.shuffled_indices = pq.shuffled_indices;
                } else {
                    state.tracks = tracks;
                    state.repeat_mode = pq.repeat_mode;
                    if pq.shuffle {
                        recalc_shuffle(&mut state);
                    }
                }

                if let Some(idx) = state.current_index {
                    current_track_to_load = state.tracks.get(idx).cloned();
                }

                drop(state);
                let _ = self.player.set_volume(persisted.player.volume).await;

                if let Some(track) = current_track_to_load {
                    if let Ok(_) = self.load_track(&track, false).await {
                        let _ = self.player.seek(persisted.player.position).await;
                    }
                }
            }
        }
    }

    pub async fn save(&self) -> Result<(), String> {
        let state = self.state.lock().await;
        let player_state = self.player.get_state().await;

        let persisted = PersistedState {
            queue: PersistedQueue {
                tracks: state.tracks.clone().iter().map(|t| t.id.clone()).collect(),
                current_index: state.current_index,
                repeat_mode: state.repeat_mode.clone(),
                shuffle: state.shuffle,
                shuffled_indices: state.shuffled_indices.clone(),
            },
            player: PersistedPlayer {
                volume: player_state.volume,
                position: player_state.position,
            },
        };

        let json = serde_json::to_string_pretty(&persisted).map_err(|e| e.to_string())?;
        if let Some(parent) = self.state_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&self.state_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_provider(&self, id: &str) -> Option<Arc<dyn LibraryProvider>> {
        self.providers.read().await.get(id).cloned()
    }

    pub async fn get_providers(&self) -> HashMap<String, Arc<dyn LibraryProvider>> {
        self.providers.read().await.clone()
    }

    pub async fn get_track(&self, track_id: &str) -> Option<Track> {
        let providers = self.providers.read().await;
        for provider in providers.values() {
            if let Ok(track) = provider.get_track(track_id).await {
                return Some(track);
            }
        }
        None
    }

    pub async fn add_provider(&self, provider: Arc<dyn LibraryProvider>) {
        self.providers
            .write()
            .await
            .insert(provider.id().to_string(), provider);
    }

    pub async fn remove_provider(&self, id: &str) {
        self.providers.write().await.remove(id);
    }

    pub async fn remove_tracks_by_provider(&self, provider_id: &str) {
        let mut state = self.state.lock().await;

        let mut new_tracks = Vec::new();
        let mut new_current_index = None;
        let mut was_playing_removed = false;
        let current_index = state.current_index;

        for (i, track) in state.tracks.drain(..).enumerate() {
            if track.provider_id.as_deref() != Some(provider_id) {
                new_tracks.push(track);
                if Some(i) == current_index {
                    new_current_index = Some(new_tracks.len() - 1);
                }
            } else if Some(i) == current_index {
                was_playing_removed = true;
            }
        }

        state.tracks = new_tracks;
        state.current_index = new_current_index;

        if state.tracks.is_empty() {
            state.current_index = None;
            state.shuffled_indices.clear();
        } else if state.shuffle {
            recalc_shuffle(&mut state);
        }

        drop(state);

        if was_playing_removed {
            let _ = self.player.stop().await;
        }
        let _ = self.save().await;
    }

    pub async fn add_track(&self, track: Track) {
        let mut state = self.state.lock().await;
        state.tracks.push(track);
        if state.shuffle {
            let len = state.tracks.len();
            state.shuffled_indices.push(len - 1);
        }
        drop(state);
        let _ = self.save().await;
    }

    pub async fn add_tracks(&self, tracks: Vec<Track>) {
        let mut state = self.state.lock().await;
        state.tracks.extend(tracks);
        if state.shuffle {
            let len = state.tracks.len();
            state.shuffled_indices.push(len - 1);
        }
        drop(state);
        let _ = self.save().await;
    }

    pub async fn add_next(&self, track: Track) {
        let mut state = self.state.lock().await;
        if let Some(curr) = state.current_index {
            if state.shuffle {
                state.tracks.push(track);
                let len = state.tracks.len();
                state.shuffled_indices.push(len - 1);
            } else {
                state.tracks.insert(curr + 1, track);
            }
        } else {
            state.tracks.push(track);
            if state.shuffle {
                let len = state.tracks.len();
                state.shuffled_indices.push(len - 1);
            }
        }
        drop(state);
        let _ = self.save().await;
    }

    pub async fn remove(&self, index: usize) {
        let mut state = self.state.lock().await;
        if index < state.tracks.len() {
            state.tracks.remove(index);
            if state.shuffle {
                recalc_shuffle(&mut state);
            }

            if let Some(curr) = state.current_index {
                if index < curr {
                    state.current_index = Some(curr - 1);
                } else if index == curr {
                    if curr > 0 {
                        state.current_index = Some(curr - 1);
                    } else {
                        state.current_index = None;
                    }
                }
            }
        }
        drop(state);
        let _ = self.save().await;
    }

    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.tracks.clear();
        state.shuffled_indices.clear();
        state.current_index = None;
        drop(state);
        let _ = self.save().await;
    }

    pub async fn toggle_shuffle(&self) -> bool {
        let mut state = self.state.lock().await;
        state.shuffle = !state.shuffle;
        if state.shuffle {
            recalc_shuffle(&mut state);
        } else {
            state.shuffled_indices.clear();
        }
        let res = state.shuffle;
        drop(state);
        let _ = self.save().await;
        res
    }

    pub async fn set_repeat(&self, mode: RepeatMode) {
        let mut state = self.state.lock().await;
        state.repeat_mode = mode;
        drop(state);
        let _ = self.save().await;
    }

    pub async fn play_now(&self, track: Track) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.tracks.clear();
        state.shuffled_indices.clear();
        state.tracks.push(track.clone());
        state.current_index = Some(0);
        if state.shuffle {
            state.shuffled_indices = vec![0];
        }
        drop(state);

        let res = self.load_track(&track, true).await;
        let _ = self.save().await;
        res
    }

    pub async fn play_index(&self, index: usize) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if index < state.tracks.len() {
            state.current_index = Some(index);
            let track = state.tracks[index].clone();
            drop(state);
            let res = self.load_track(&track, true).await;
            let _ = self.save().await;
            res
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    pub async fn on_playback_ended(&self) -> Result<(), String> {
        let state = self.state.lock().await;
        if matches!(state.repeat_mode, RepeatMode::One) {
            if let Some(curr) = state.current_index {
                if curr < state.tracks.len() {
                    let track = state.tracks[curr].clone();
                    drop(state);
                    return self.load_track(&track, true).await;
                }
            }
        }
        drop(state);
        self.next().await
    }

    pub async fn next(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;

        let next_idx = if state.shuffle {
            if let Some(curr_raw) = state.current_index {
                if let Some(pos_in_shuffle) =
                    state.shuffled_indices.iter().position(|&r| r == curr_raw)
                {
                    if pos_in_shuffle + 1 < state.shuffled_indices.len() {
                        Some(state.shuffled_indices[pos_in_shuffle + 1])
                    } else if matches!(state.repeat_mode, RepeatMode::All) {
                        Some(state.shuffled_indices[0])
                    } else {
                        None
                    }
                } else {
                    state.shuffled_indices.first().cloned()
                }
            } else {
                state.shuffled_indices.first().cloned()
            }
        } else if let Some(curr) = state.current_index {
            if curr + 1 < state.tracks.len() {
                Some(curr + 1)
            } else if matches!(state.repeat_mode, RepeatMode::All) && !state.tracks.is_empty() {
                Some(0)
            } else {
                None
            }
        } else if !state.tracks.is_empty() {
            Some(0)
        } else {
            None
        };

        if let Some(idx) = next_idx {
            state.current_index = Some(idx);
            let track = state.tracks[idx].clone();
            drop(state);
            let res = self.load_track(&track, true).await;
            let _ = self.save().await;
            res
        } else {
            Ok(())
        }
    }

    pub async fn prev(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if let Some(curr) = state.current_index {
            if curr > 0 {
                state.current_index = Some(curr - 1);
                let track = state.tracks[curr - 1].clone();
                drop(state);
                let res = self.load_track(&track, true).await;
                let _ = self.save().await;
                return res;
            }
        }
        Ok(())
    }

    async fn load_track(&self, track: &Track, auto_play: bool) -> Result<(), String> {
        let providers = self.providers.read().await;

        if let Some(pid) = &track.provider_id {
            if let Some(provider) = providers.get(pid) {
                if let Ok(stream) = provider.resolve_stream(&track.id).await {
                    return self.player.load(stream, auto_play).await;
                }
            }
        }

        for (pid, provider) in providers.iter() {
            if track.id.starts_with(pid) {
                let real_id = track
                    .id
                    .strip_prefix(&format!("{}:", pid))
                    .unwrap_or(&track.id);
                if let Ok(stream) = provider.resolve_stream(real_id).await {
                    return self.player.load(stream, auto_play).await;
                }
            }

            if let Ok(stream) = provider.resolve_stream(&track.id).await {
                return self.player.load(stream, auto_play).await;
            }
        }

        Err("Could not resolve track in any provider".to_string())
    }

    pub async fn get_queue(&self) -> Queue {
        let state = self.state.lock().await;
        Queue {
            id: "main".to_string(),
            tracks: state.tracks.clone(),
            current_index: state.current_index.unwrap_or(0) as u32,
            shuffle: state.shuffle,
            repeat: state.repeat_mode.clone(),
        }
    }

    pub async fn current_track(&self) -> Option<Track> {
        let state = self.state.lock().await;
        if let Some(idx) = state.current_index {
            state.tracks.get(idx).cloned()
        } else {
            None
        }
    }
}

fn recalc_shuffle(state: &mut QueueState) {
    let mut indices: Vec<usize> = (0..state.tracks.len()).collect();
    let mut rng = rand::rng();
    indices.shuffle(&mut rng);
    state.shuffled_indices = indices;
}
