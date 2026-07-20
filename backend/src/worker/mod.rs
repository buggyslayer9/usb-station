use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, error};

use crate::AppState;

pub mod flash_worker;

/// Spawn background workers for the application.
pub async fn start_workers(state: AppState) {
    let max_concurrent = state.config.flash.max_concurrent as usize;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // Flash worker pool
    for i in 0..max_concurrent {
        let state = state.clone();
        let sem = semaphore.clone();
        tokio::spawn(async move {
            flash_worker::run_worker(i, state, sem).await;
        });
    }

    info!(workers = max_concurrent, "Background workers started");
}
