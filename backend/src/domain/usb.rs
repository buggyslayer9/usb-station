use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub id: Uuid,
    pub device_path: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub capacity_bytes: u64,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub is_mounted: bool,
    pub is_readonly: bool,
    pub is_system_disk: bool,
    pub health: DiskHealth,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiskHealth {
    Good,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsbEvent {
    Inserted(UsbDevice),
    Removed(Uuid),
    Changed(UsbDevice),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDeviceInfo {
    pub device_path: String,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub size_sectors: u64,
    pub sector_size: u64,
    pub is_removable: bool,
    pub is_readonly: bool,
}
