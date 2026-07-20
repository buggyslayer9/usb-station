use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashBatch {
    pub id: Uuid,
    pub name: String,
    pub mode: BatchMode,
    pub status: BatchStatus,
    pub jobs: Vec<Uuid>,
    pub total_jobs: u32,
    pub completed_jobs: u32,
    pub failed_jobs: u32,
    pub max_concurrent: u32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchMode {
    /// 1 ISO → multiple USBs simultaneously
    Clone,
    /// N ISOs → N USBs, paired by order
    Sequential,
    /// System auto-matches ISOs to USBs by capacity
    SmartAssign,
    /// User manually queues individual jobs in a batch
    ManualQueue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchStatus {
    Draft,
    Queued,
    Running,
    Completed,
    PartiallyFailed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchCreateRequest {
    pub name: String,
    pub mode: BatchMode,
    pub iso_ids: Vec<Uuid>,
    pub usb_ids: Vec<Uuid>,
    pub max_concurrent: Option<u32>,
    pub verify: Option<bool>,
    /// For SmartAssign mode: optional priority rules
    pub priority_rules: Option<Vec<SmartAssignRule>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmartAssignRule {
    pub iso_tag: Option<String>,
    pub usb_min_bytes: Option<u64>,
    pub usb_max_bytes: Option<u64>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub batch_id: Uuid,
    pub mode: BatchMode,
    pub status: BatchStatus,
    pub total_jobs: u32,
    pub completed_jobs: u32,
    pub failed_jobs: u32,
    pub running_jobs: u32,
    pub pending_jobs: u32,
    pub overall_progress: f64,
    pub jobs: Vec<super::flash::FlashJobProgress>,
}

impl FlashBatch {
    pub fn overall_progress(&self) -> f64 {
        if self.total_jobs == 0 {
            return 0.0;
        }
        (self.completed_jobs + self.failed_jobs) as f64 / self.total_jobs as f64 * 100.0
    }
}
