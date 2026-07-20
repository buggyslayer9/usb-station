use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error};

use crate::AppState;

/// Background worker that processes queued flash jobs.
pub async fn run_worker(id: usize, state: AppState, semaphore: Arc<Semaphore>) {
    info!("Flash worker {} started", id);

    loop {
        let _permit = semaphore.acquire().await;

        // TODO: pull next job from queue
        // For now, sleep and retry
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
