use std::path::PathBuf;
use uuid::Uuid;
use tracing::{info, warn};

use crate::domain::usb::*;

pub struct UsbService;

impl UsbService {
    pub async fn scan_devices() -> Vec<UsbDevice> {
        let mut devices = Vec::new();

        match std::fs::read_dir("/sys/block") {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();

                    let removable_path = entry.path().join("removable");
                    let is_removable = std::fs::read_to_string(&removable_path)
                        .map(|s| s.trim() == "1")
                        .unwrap_or(false);

                    if !is_removable || (!name_str.starts_with("sd") && !name_str.starts_with("nvme")) {
                        continue;
                    }

                    let device_path = format!("/dev/{}", name_str);
                    let base = entry.path();

                    let size_path = base.join("size");
                    let capacity = std::fs::read_to_string(&size_path)
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok())
                        .map(|sectors| sectors * 512)
                        .unwrap_or(0);

                    let vendor = std::fs::read_to_string(base.join("device/vendor"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let model = std::fs::read_to_string(base.join("device/model"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let serial = std::fs::read_to_string(base.join("serial"))
                        .ok()
                        .map(|s| s.trim().to_string());

                    let rotational = std::fs::read_to_string(base.join("queue/rotational"))
                        .ok()
                        .and_then(|s| s.trim().parse::<u8>().ok())
                        .unwrap_or(1);

                    let mounts = Self::read_mounts();
                    let mount_info = mounts.iter()
                        .find(|(dev, _)| dev == &device_path);

                    let filesystem = mount_info.and_then(|(dev, _)| {
                        std::fs::read_to_string("/proc/mounts").ok().and_then(|content| {
                            content.lines().find_map(|line| {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 3 && parts[0] == *dev { Some(parts[2].to_string()) } else { None }
                            })
                        })
                    });

                    devices.push(UsbDevice {
                        id: Uuid::new_v4(),
                        device_path,
                        vendor,
                        model,
                        serial,
                        capacity_bytes: capacity,
                        filesystem,
                        mount_point: mount_info.map(|(_, mnt)| mnt.clone()),
                        is_mounted: mount_info.is_some(),
                        is_readonly: false,
                        is_system_disk: Self::is_system_disk(&name_str, &mounts),
                        health: if rotational == 0 { DiskHealth::Good } else { DiskHealth::Good },
                        inserted_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    });
                }
            }
            Err(e) => warn!("Failed to read /sys/block: {}", e),
        }

        devices
    }

    pub fn get_device_info(path: &str) -> Option<UsbDeviceInfo> {
        let dev_name = path.trim_start_matches("/dev/");
        if dev_name.is_empty() || dev_name.contains('/') {
            return None;
        }

        let base_name = if dev_name.starts_with("nvme") {
            dev_name.trim_end_matches(|c: char| c.is_ascii_digit() || c == 'p')
        } else {
            dev_name.trim_end_matches(|c: char| c.is_ascii_digit())
        };

        let sys_base = PathBuf::from("/sys/block").join(base_name);
        if !sys_base.exists() {
            return None;
        }

        let sys_path = if base_name == dev_name {
            sys_base.clone()
        } else {
            sys_base.join(dev_name)
        };

        let size_sectors = std::fs::read_to_string(sys_path.join("size"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        let sector_size = std::fs::read_to_string(sys_path.join("queue/hw_sector_size"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(512);

        let is_removable = std::fs::read_to_string(sys_base.join("removable"))
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        let vendor = std::fs::read_to_string(sys_base.join("device/vendor"))
            .ok()
            .map(|s| s.trim().to_string());

        let model = std::fs::read_to_string(sys_base.join("device/model"))
            .ok()
            .map(|s| s.trim().to_string());

        let serial = std::fs::read_to_string(sys_base.join("serial"))
            .ok()
            .map(|s| s.trim().to_string());

        let is_readonly = std::fs::read_to_string(sys_path.join("ro"))
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        Some(UsbDeviceInfo {
            device_path: path.to_string(),
            vendor,
            model,
            serial,
            size_sectors,
            sector_size,
            is_removable,
            is_readonly,
        })
    }

    pub async fn eject(device_path: &str) -> Result<(), String> {
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

    fn is_system_disk(name: &str, mounts: &[(String, String)]) -> bool {
        if name == "sda" || name.starts_with("nvme0n1") || name.starts_with("mmcblk0") {
            return true;
        }
        let dev = format!("/dev/{}", name);
        mounts.iter().any(|(d, m)| {
            d.starts_with(&dev) && (m == "/" || m == "/boot" || m.starts_with("/boot/") || m.contains("docker"))
        })
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
