use crate::queue::QueueManager;
use crate::util::discord::DiscordRpc;
use crate::util::lastfm::LastFmClient;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub queue: Arc<QueueManager>,
    pub lastfm: Arc<Mutex<Option<LastFmClient>>>,
    pub discord: Arc<Mutex<DiscordRpc>>,
}

impl AppState {
    pub fn new(queue: Arc<QueueManager>, lastfm: Option<LastFmClient>, discord: DiscordRpc) -> Self {
        Self {
            queue,
            lastfm: Arc::new(Mutex::new(lastfm)),
            discord: Arc::new(Mutex::new(discord)),
        }
    }
}
