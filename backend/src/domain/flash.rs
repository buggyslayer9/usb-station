use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashJob {
    pub id: Uuid,
    pub iso_id: Uuid,
    pub usb_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub status: FlashStatus,
    pub progress_percent: f64,
    pub speed_bytes_per_sec: u64,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub eta_seconds: Option<u64>,
    pub verify: bool,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlashStatus {
    Pending,
    Queued,
    Flashing,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashJobProgress {
    pub job_id: Uuid,
    pub status: FlashStatus,
    pub progress_percent: f64,
    pub speed_bytes_per_sec: u64,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashRequest {
    pub iso_id: Uuid,
    pub usb_id: Uuid,
    pub verify: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashEngineInfo {
    pub engine: String,
    pub available: bool,
    pub version: Option<String>,
}

pub enum FlashEngine {
    Dd,
    Bmaptool,
    Xzcat,
    NativeRust,
}
