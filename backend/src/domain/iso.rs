use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoImage {
    pub id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub detected_os: Option<String>,
    pub detected_version: Option<String>,
    pub boot_mode: Option<BootMode>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub is_favorite: bool,
    pub scanned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BootMode {
    Bios,
    Uefi,
    Both,
    Unknown,
}

impl IsoImage {
    pub fn fits_on(&self, capacity_bytes: u64) -> bool {
        self.file_size_bytes < capacity_bytes
    }
}
