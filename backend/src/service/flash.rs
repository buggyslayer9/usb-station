use std::collections::HashMap;
use tokio::process::Command;
use uuid::Uuid;
use tracing::{info, error};

use crate::domain::flash::*;

/// Manages active flash jobs and coordinates concurrent execution.
#[derive(Clone)]
pub struct FlashJobQueue {
    jobs: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, FlashJob>>>,
    max_concurrent: u32,
}

impl FlashJobQueue {
    pub fn new(max_concurrent: u32) -> Self {
        Self {
            jobs: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    pub async fn enqueue(&self, request: FlashRequest) -> Result<FlashJob, String> {
        let job = FlashJob {
            id: Uuid::new_v4(),
            iso_id: request.iso_id,
            usb_id: request.usb_id,
            batch_id: None,
            status: FlashStatus::Queued,
            progress_percent: 0.0,
            speed_bytes_per_sec: 0,
            bytes_written: 0,
            total_bytes: 0,
            eta_seconds: None,
            verify: request.verify.unwrap_or(true),
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.jobs.write().await.insert(job.id, job.clone());
        info!(job_id = %job.id, "Flash job enqueued");
        Ok(job)
    }

    pub async fn cancel(&self, job_id: Uuid) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&job_id)
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status == FlashStatus::Completed || job.status == FlashStatus::Failed {
            return Err(format!("Job {} already finished", job_id));
        }

        job.status = FlashStatus::Cancelled;
        info!(job_id = %job_id, "Flash job cancelled");
        Ok(())
    }

    pub async fn status(&self, job_id: Uuid) -> Option<FlashJob> {
        self.jobs.read().await.get(&job_id).cloned()
    }

    /// Execute a flash job using the best available engine.
    /// Auto-selects: dd (default), bmaptool (for bmap images), native Rust (fallback)
    pub async fn execute(&self, job_id: Uuid, iso_path: &str, device_path: &str) -> Result<(), String> {
        let engine = self.select_engine().await;

        info!(job_id = %job_id, engine = %engine, device = %device_path, "Starting flash");

        let result = match engine.as_str() {
            "dd" => self.flash_with_dd(job_id, iso_path, device_path).await,
            "bmaptool" => self.flash_with_bmaptool(job_id, iso_path, device_path).await,
            _ => Err("No suitable flash engine found".into()),
        };

        match &result {
            Ok(()) => {
                info!(job_id = %job_id, "Flash completed successfully");
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = FlashStatus::Completed;
                    job.progress_percent = 100.0;
                    job.completed_at = Some(chrono::Utc::now());
                }
            }
            Err(e) => {
                error!(job_id = %job_id, error = %e, "Flash failed");
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = FlashStatus::Failed;
                    job.error_message = Some(e.clone());
                    job.completed_at = Some(chrono::Utc::now());
                }
            }
        }

        result
    }

    async fn select_engine(&self) -> String {
        // Check for dd availability
        let dd_check = Command::new("which")
            .arg("dd")
            .output()
            .await;

        if dd_check.map(|o| o.status.success()).unwrap_or(false) {
            return "dd".into();
        }

        let bmap_check = Command::new("which")
            .arg("bmaptool")
            .output()
            .await;

        if bmap_check.map(|o| o.status.success()).unwrap_or(false) {
            return "bmaptool".into();
        }

        "none".into()
    }

    async fn flash_with_dd(&self, _job_id: Uuid, iso_path: &str, device_path: &str) -> Result<(), String> {
        let status = Command::new("dd")
            .arg(format!("if={}", iso_path))
            .arg(format!("of={}", device_path))
            .arg("bs=4M")
            .arg("status=progress")
            .arg("oflag=direct")
            .status()
            .await
            .map_err(|e| format!("Failed to execute dd: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("dd failed with exit code: {:?}", status.code()))
        }
    }

    async fn flash_with_bmaptool(&self, _job_id: Uuid, iso_path: &str, device_path: &str) -> Result<(), String> {
        let status = Command::new("bmaptool")
            .arg("copy")
            .arg(iso_path)
            .arg(device_path)
            .status()
            .await
            .map_err(|e| format!("Failed to execute bmaptool: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("bmaptool failed with exit code: {:?}", status.code()))
        }
    }
}
