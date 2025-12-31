pub mod config;
pub mod entities;
pub mod player;

pub use config::{AppConfig, AudioBackend};
pub use entities::{Album, Artist, Track};
pub use player::PlayerState;
