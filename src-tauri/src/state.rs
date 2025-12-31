use crate::queue::QueueManager;
use std::sync::Arc;

pub struct AppState {
    pub queue: Arc<QueueManager>,
}

impl AppState {
    pub fn new(queue: Arc<QueueManager>) -> Self {
        Self { queue }
    }
}
