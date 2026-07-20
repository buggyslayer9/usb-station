use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use tracing::{info, error, warn};

use crate::service::flash::FlashCommand;
use crate::AppState;

pub async fn run_worker(
    id: usize,
    mut cmd_rx: broadcast::Receiver<FlashCommand>,
    state: AppState,
    semaphore: Arc<Semaphore>,
) {
    info!("Flash worker {} started", id);

    loop {
        let _permit = semaphore.acquire().await;
        info!(worker = id, "Waiting for flash command");

        let cmd = match cmd_rx.recv().await {
            Ok(cmd) => cmd,
            Err(broadcast::error::RecvError::Closed) => {
                info!(worker = id, "Command channel closed, shutting down");
                return;
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!(worker = id, lagged = n, "Worker receiver lagged");
                continue;
            }
        };

        match cmd {
            FlashCommand::Execute { job_id, iso_path, device_path } => {
                let claimed = state.job_queue.claim_job(job_id).await;
                if !claimed {
                    info!(worker = id, job_id = %job_id, "Job already claimed by another worker");
                    continue;
                }

                info!(
                    worker = id,
                    job_id = %job_id,
                    iso = %iso_path,
                    device = %device_path,
                    "Starting flash execution"
                );

                let result = state.job_queue.execute(job_id, &iso_path, &device_path).await;

                match result {
                    Ok(()) => {
                        info!(worker = id, job_id = %job_id, "Flash completed successfully");
                    }
                    Err(e) => {
                        error!(worker = id, job_id = %job_id, error = %e, "Flash failed");
                    }
                }
            }
            FlashCommand::Cancel(job_id) => {
                info!(worker = id, job_id = %job_id, "Received cancel command for job");
            }
        }
    }
}
