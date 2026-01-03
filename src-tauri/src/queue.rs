use crate::models::{
    entities::{PlayerEvent, Track},
    player::{Queue, RepeatMode},
};
use crate::traits::{AudioEngine, LibraryProvider};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct QueueManager {
    state: Mutex<QueueState>,
    pub player: Box<dyn AudioEngine>,
    providers: Arc<RwLock<HashMap<String, Arc<dyn LibraryProvider>>>>,
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
    ) -> Arc<Self> {
        let providers = Arc::new(RwLock::new(providers));
        let qm = Arc::new(Self {
            state: Mutex::new(QueueState::default()),
            player,
            providers: providers.clone(),
        });

        let qm_clone = qm.clone();
        tokio::spawn(async move {
            let mut rx = qm_clone.player.subscribe();
            while let Ok(event) = rx.recv().await {
                if let PlayerEvent::Ended = event {
                    let _ = qm_clone.on_playback_ended().await;
                }
            }
        });

        qm
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

    pub async fn add_track(&self, track: Track) {
        let mut state = self.state.lock().await;
        state.tracks.push(track);
        if state.shuffle {
            let len = state.tracks.len();
            state.shuffled_indices.push(len - 1);
        }
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
    }

    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.tracks.clear();
        state.shuffled_indices.clear();
        state.current_index = None;
    }

    pub async fn toggle_shuffle(&self) -> bool {
        let mut state = self.state.lock().await;
        state.shuffle = !state.shuffle;
        if state.shuffle {
            recalc_shuffle(&mut state);
        } else {
            state.shuffled_indices.clear();
        }
        state.shuffle
    }

    pub async fn set_repeat(&self, mode: RepeatMode) {
        let mut state = self.state.lock().await;
        state.repeat_mode = mode;
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

        self.load_track(&track).await
    }

    pub async fn play_index(&self, index: usize) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if index < state.tracks.len() {
            state.current_index = Some(index);
            let track = state.tracks[index].clone();
            drop(state);
            self.load_track(&track).await
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
                    return self.load_track(&track).await;
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
            self.load_track(&track).await
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
                return self.load_track(&track).await;
            }
        }
        Ok(())
    }

    async fn load_track(&self, track: &Track) -> Result<(), String> {
        let providers = self.providers.read().await;

        if let Some(pid) = &track.provider_id {
            if let Some(provider) = providers.get(pid) {
                if let Ok(stream) = provider.resolve_stream(&track.id).await {
                    return self.player.load(stream, true).await;
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
                    return self.player.load(stream, true).await;
                }
            }

            if let Ok(stream) = provider.resolve_stream(&track.id).await {
                return self.player.load(stream, true).await;
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
}

fn recalc_shuffle(state: &mut QueueState) {
    let mut indices: Vec<usize> = (0..state.tracks.len()).collect();
    let mut rng = rand::rng();
    indices.shuffle(&mut rng);
    state.shuffled_indices = indices;
}
