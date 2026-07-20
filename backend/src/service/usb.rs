use uuid::Uuid;
use tracing::{info, warn};

use crate::domain::usb::*;

/// Service for USB device detection, monitoring, and safe ejection.
pub struct UsbService;

impl UsbService {
    /// Scan all currently connected USB storage devices.
    pub async fn scan_devices() -> Vec<UsbDevice> {
        let mut devices = Vec::new();

        match std::fs::read_dir("/sys/block") {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();

                    // Skip non-removable devices
                    let removable_path = entry.path().join("removable");
                    let is_removable = std::fs::read_to_string(&removable_path)
                        .map(|s| s.trim() == "1")
                        .unwrap_or(false);

                    if !is_removable || (!name_str.starts_with("sd") && !name_str.starts_with("nvme")) {
                        continue;
                    }

                    let device_path = format!("/dev/{}", name_str);
                    let size_path = entry.path().join("size");
                    let capacity = std::fs::read_to_string(&size_path)
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .map(|sectors| sectors * 512)
                        .unwrap_or(0);

                    let vendor = std::fs::read_to_string(entry.path().join("device/vendor"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let model = std::fs::read_to_string(entry.path().join("device/model"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let serial = std::fs::read_to_string(entry.path().join("serial"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    // Check mounts
                    let mounts = Self::read_mounts();
                    let mount_info = mounts.iter()
                        .find(|(dev, _)| dev == &device_path);

                    devices.push(UsbDevice {
                        id: Uuid::new_v4(),
                        device_path,
                        vendor,
                        model,
                        serial,
                        capacity_bytes: capacity,
                        filesystem: None,
                        mount_point: mount_info.map(|(_, mnt)| mnt.clone()),
                        is_mounted: mount_info.is_some(),
                        is_readonly: false,
                        is_system_disk: Self::is_system_disk(&name_str),
                        health: DiskHealth::Good,
                        inserted_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    });
                }
            }
            Err(e) => warn!("Failed to read /sys/block: {}", e),
        }

        devices
    }

    /// Safely unmount and eject a USB device.
    pub async fn eject(device_path: &str) -> Result<(), String> {
        // Find mount point
        let mounts = Self::read_mounts();
        if let Some((_, mount_point)) = mounts.iter().find(|(dev, _)| dev == &device_path) {
            let status = tokio::process::Command::new("umount")
                .arg(mount_point)
                .status()
                .await
                .map_err(|e| format!("Failed to unmount: {}", e))?;

            if !status.success() {
                return Err(format!("Unmount failed for {}", mount_point));
            }
        }

        // Eject the device
        let status = tokio::process::Command::new("eject")
            .arg(device_path)
            .status()
            .await
            .map_err(|e| format!("Failed to eject: {}", e))?;

        if status.success() {
            info!(device = %device_path, "Device ejected safely");
            Ok(())
        } else {
            Err(format!("Eject failed for {}", device_path))
        }
    }

    fn is_system_disk(name: &str) -> bool {
        // Heuristic: boot/root partitions typically on sda or nvme0n1
        name == "sda" || name.starts_with("nvme0n1") || name.starts_with("mmcblk0")
    }

    fn read_mounts() -> Vec<(String, String)> {
        let content = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
        content.lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].starts_with("/dev/") {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect()
    }
}
