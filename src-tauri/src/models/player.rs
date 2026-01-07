use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub paused: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
    pub exclusive: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub id: String,
    pub tracks: Vec<super::entities::Track>,
    pub current_index: u32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PersistedQueue {
    pub tracks: Vec<String>,
    pub current_index: Option<usize>,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub shuffled_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PersistedPlayer {
    pub volume: f32,
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct PersistedState {
    pub queue: PersistedQueue,
    pub player: PersistedPlayer,
}
