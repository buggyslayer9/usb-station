use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use tracing::{info, error};

use crate::service::flash::FlashCommand;
use crate::AppState;

pub mod flash_worker;

pub async fn start_workers(state: AppState, cmd_rx: broadcast::Receiver<FlashCommand>) {
    let max_concurrent = state.config.flash.max_concurrent as usize;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    for i in 0..max_concurrent {
        let state = state.clone();
        let sem = semaphore.clone();
        let rx = cmd_rx.resubscribe();

        tokio::spawn(async move {
            flash_worker::run_worker(i, rx, state, sem).await;
        });
    }

    info!(workers = max_concurrent, "Background workers started");
}
