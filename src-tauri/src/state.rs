use crate::queue::QueueManager;
use crate::util::lastfm::LastFmClient;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub queue: Arc<QueueManager>,
    pub lastfm: Arc<Mutex<Option<LastFmClient>>>,
}

impl AppState {
    pub fn new(queue: Arc<QueueManager>, lastfm: Option<LastFmClient>) -> Self {
        Self {
            queue,
            lastfm: Arc::new(Mutex::new(lastfm)),
        }
    }
}
