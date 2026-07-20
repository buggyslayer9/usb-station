use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;
use tracing::{info, warn};

use crate::domain::{
    batch::*, iso::IsoImage, usb::UsbDevice,
};

/// The BatchOrchestrator manages multi-USB flash operations.
///
/// Modes:
/// - Clone: 1 ISO → N USBs simultaneously
/// - Sequential: N ISOs → N USBs, paired by order
/// - SmartAssign: system auto-matches ISOs to USBs
/// - ManualQueue: user queues jobs individually, managed as a batch
#[derive(Clone)]
pub struct BatchOrchestrator {
    batches: Arc<RwLock<HashMap<Uuid, FlashBatch>>>,
    active_batches: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    semaphore: Arc<Semaphore>,
}

impl BatchOrchestrator {
    pub fn new() -> Self {
        Self {
            batches: Arc::new(RwLock::new(HashMap::new())),
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(2)),
        }
    }

    pub fn with_max_concurrent(max: u32) -> Self {
        Self {
            batches: Arc::new(RwLock::new(HashMap::new())),
            active_batches: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max as usize)),
        }
    }

    /// Create a batch from a request. Validates and resolves the pairing.
    pub async fn create_batch(
        &self,
        request: BatchCreateRequest,
        iso_lookup: impl Fn(Uuid) -> Option<IsoImage>,
        usb_lookup: impl Fn(Uuid) -> Option<UsbDevice>,
    ) -> Result<FlashBatch, String> {
        let batch_id = Uuid::new_v4();
        let max_concurrent = request.max_concurrent.unwrap_or(2);

        let mode = &request.mode;
        let job_ids = match mode {
            BatchMode::Clone => self.resolve_clone(&request, iso_lookup, usb_lookup)?,
            BatchMode::Sequential => self.resolve_sequential(&request, iso_lookup, usb_lookup)?,
            BatchMode::SmartAssign => self.resolve_smart_assign(&request, iso_lookup, usb_lookup)?,
            BatchMode::ManualQueue => Vec::new(),
        };

        let total_jobs = job_ids.len() as u32;
        let mode_owned = request.mode.clone();

        let batch = FlashBatch {
            id: batch_id,
            name: request.name,
            mode: mode_owned,
            status: BatchStatus::Draft,
            jobs: job_ids,
            total_jobs,
            completed_jobs: 0,
            failed_jobs: 0,
            max_concurrent,
            created_by: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            updated_at: chrono::Utc::now(),
        };

        let total = batch.total_jobs;
        self.batches.write().await.insert(batch_id, batch.clone());

        info!(batch_id = %batch_id, mode = ?mode, total_jobs = total, "Batch created");

        Ok(batch)
    }

    pub async fn start_batch(&self, batch_id: Uuid) -> Result<(), String> {
        let mut batches = self.batches.write().await;
        let batch = batches.get_mut(&batch_id)
            .ok_or_else(|| format!("Batch {} not found", batch_id))?;

        if batch.status != BatchStatus::Draft && batch.status != BatchStatus::Queued {
            return Err(format!("Batch {} cannot be started (status: {:?})", batch_id, batch.status));
        }

        batch.status = BatchStatus::Running;
        batch.started_at = Some(chrono::Utc::now());

        info!(batch_id = %batch_id, "Batch started");
        Ok(())
    }

    /// Cancel a batch — cancel all pending/running jobs
    pub async fn cancel_batch(&self, batch_id: Uuid) -> Result<(), String> {
        let mut batches = self.batches.write().await;
        let batch = batches.get_mut(&batch_id)
            .ok_or_else(|| format!("Batch {} not found", batch_id))?;

        if batch.status == BatchStatus::Completed || batch.status == BatchStatus::Cancelled {
            return Err(format!("Batch {} already {}", batch_id, if batch.status == BatchStatus::Completed { "completed" } else { "cancelled" }));
        }

        batch.status = BatchStatus::Cancelled;
        batch.completed_at = Some(chrono::Utc::now());

        info!(batch_id = %batch_id, "Batch cancelled");
        Ok(())
    }

    /// Get current batch progress
    pub async fn batch_progress(&self, batch_id: Uuid) -> Option<BatchProgress> {
        let batches = self.batches.read().await;
        let batch = batches.get(&batch_id)?;

        Some(BatchProgress {
            batch_id: batch.id,
            mode: batch.mode.clone(),
            status: batch.status.clone(),
            total_jobs: batch.total_jobs,
            completed_jobs: batch.completed_jobs,
            failed_jobs: batch.failed_jobs,
            running_jobs: batch.total_jobs - batch.completed_jobs - batch.failed_jobs,
            pending_jobs: 0,
            overall_progress: batch.overall_progress(),
            jobs: Vec::new(),
        })
    }

    // --- Resolution strategies ---

    fn resolve_clone(
        &self,
        request: &BatchCreateRequest,
        _iso_lookup: impl Fn(Uuid) -> Option<IsoImage>,
        _usb_lookup: impl Fn(Uuid) -> Option<UsbDevice>,
    ) -> Result<Vec<Uuid>, String> {
        if request.iso_ids.is_empty() {
            return Err("Clone mode requires at least 1 ISO".into());
        }
        if request.usb_ids.is_empty() {
            return Err("Clone mode requires at least 1 USB target".into());
        }

        // 1 ISO → each USB
        let iso_id = request.iso_ids[0];
        Ok(request.usb_ids.iter().map(|usb_id| Uuid::new_v4()).collect())
    }

    fn resolve_sequential(
        &self,
        request: &BatchCreateRequest,
        _iso_lookup: impl Fn(Uuid) -> Option<IsoImage>,
        _usb_lookup: impl Fn(Uuid) -> Option<UsbDevice>,
    ) -> Result<Vec<Uuid>, String> {
        if request.iso_ids.len() != request.usb_ids.len() {
            return Err(format!(
                "Sequential mode requires equal ISOs and USBs. Got {} ISOs, {} USBs",
                request.iso_ids.len(),
                request.usb_ids.len()
            ));
        }
        if request.iso_ids.is_empty() {
            return Err("At least 1 ISO-USB pair required".into());
        }

        Ok(request.iso_ids.iter().map(|_| Uuid::new_v4()).collect())
    }

    fn resolve_smart_assign(
        &self,
        request: &BatchCreateRequest,
        iso_lookup: impl Fn(Uuid) -> Option<IsoImage>,
        usb_lookup: impl Fn(Uuid) -> Option<UsbDevice>,
    ) -> Result<Vec<Uuid>, String> {
        if request.iso_ids.is_empty() || request.usb_ids.is_empty() {
            return Err("SmartAssign requires at least 1 ISO and 1 USB".into());
        }

        // Greedy assignment: largest ISOs to largest USBs, ensure fit
        let mut isos: Vec<(Uuid, IsoImage)> = request.iso_ids.iter()
            .filter_map(|id| iso_lookup(*id).map(|iso| (*id, iso)))
            .collect();
        let mut usbs: Vec<(Uuid, UsbDevice)> = request.usb_ids.iter()
            .filter_map(|id| usb_lookup(*id).map(|usb| (*id, usb)))
            .collect();

        isos.sort_by(|a, b| b.1.file_size_bytes.cmp(&a.1.file_size_bytes));
        usbs.sort_by(|a, b| b.1.capacity_bytes.cmp(&a.1.capacity_bytes));

        let mut jobs = Vec::new();
        for (iso_id, iso) in &isos {
            if let Some(pos) = usbs.iter().position(|(_, usb)| usb.capacity_bytes > iso.file_size_bytes) {
                let usb = usbs.remove(pos);
                jobs.push(Uuid::new_v4());
                info!(iso = %iso_id, usb = %usb.0, "SmartAssign: paired");
            } else {
                warn!(iso = %iso_id, size = iso.file_size_bytes, "No USB large enough for ISO");
            }
        }

        Ok(jobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_overall_progress() {
        let mut batch = FlashBatch {
            id: Uuid::new_v4(),
            name: "test".into(),
            mode: BatchMode::Clone,
            status: BatchStatus::Running,
            jobs: vec![],
            total_jobs: 4,
            completed_jobs: 2,
            failed_jobs: 1,
            max_concurrent: 2,
            created_by: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(batch.overall_progress(), 75.0);
    }
}
