use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{info, debug};
use uuid::Uuid;

use crate::domain::usb::{UsbDevice, UsbEvent, DiskHealth};

#[derive(Debug, Clone)]
struct SnapshotEntry {
    name: String,
    size_sectors: u64,
    removable: bool,
}

#[derive(Clone)]
pub struct UsbMonitor {
    tx: broadcast::Sender<UsbEvent>,
}

impl UsbMonitor {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            Self::polling_monitor(tx_clone).await;
        });

        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UsbEvent> {
        self.tx.subscribe()
    }

    pub fn current_devices() -> Vec<UsbDevice> {
        scan_removable_devices()
    }

    async fn polling_monitor(tx: broadcast::Sender<UsbEvent>) {
        let mut previous: HashMap<String, SnapshotEntry> = HashMap::new();
        let mut tick = interval(Duration::from_secs(2));

        loop {
            tick.tick().await;
            let current = scan_sys_block_snapshot();

            for (name, prev) in &previous {
                if !current.contains_key(name) && prev.removable {
                    if let Some(device) = scan_device(name) {
                        debug!(device = %name, "USB device removed");
                        let _ = tx.send(UsbEvent::Removed(device.id));
                    }
                }
            }

            for (name, curr) in &current {
                if !curr.removable {
                    continue;
                }
                if let Some(prev) = previous.get(name) {
                    if prev.size_sectors != curr.size_sectors {
                        if let Some(device) = scan_device(name) {
                            debug!(device = %name, "USB device changed");
                            let _ = tx.send(UsbEvent::Changed(device));
                        }
                    }
                } else {
                    if let Some(device) = scan_device(name) {
                        info!(device = %name, vendor = ?device.vendor, model = ?device.model, "USB device inserted");
                        let _ = tx.send(UsbEvent::Inserted(device));
                    }
                }
            }

            previous = current;
        }
    }
}

fn scan_sys_block_snapshot() -> HashMap<String, SnapshotEntry> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir("/sys/block") else { return map };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let removable = std::fs::read_to_string(entry.path().join("removable"))
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(false);
        let size_sectors = std::fs::read_to_string(entry.path().join("size"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        map.insert(name.clone(), SnapshotEntry { name, size_sectors, removable });
    }
    map
}

fn scan_removable_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/block") else { return devices };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let removable = std::fs::read_to_string(entry.path().join("removable"))
            .map(|s| s.trim() == "1")
            .unwrap_or(false);
        if !removable || (!name.starts_with("sd") && !name.starts_with("nvme")) {
            continue;
        }
        if let Some(device) = scan_device(&name) {
            devices.push(device);
        }
    }
    devices
}

fn scan_device(name: &str) -> Option<UsbDevice> {
    let base = PathBuf::from("/sys/block").join(name);
    let device_path = format!("/dev/{}", name);

    let capacity = std::fs::read_to_string(base.join("size"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|sectors| sectors * 512)
        .unwrap_or(0);

    let vendor = read_sysfs_attr(&base.join("device/vendor"));
    let model = read_sysfs_attr(&base.join("device/model"));
    let serial = read_sysfs_attr(&base.join("serial"));

    let mounts = read_mounts();
    let mount_info = mounts.iter().find(|(dev, _)| dev == &device_path);

    let filesystem = mount_info.and_then(|(dev, _)| {
        std::fs::read_to_string("/proc/mounts").ok().and_then(|content| {
            content.lines().find_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == *dev { Some(parts[2].to_string()) } else { None }
            })
        })
    });

    let now = chrono::Utc::now();
    Some(UsbDevice {
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
        is_system_disk: check_system_disk(name, &mounts),
        health: DiskHealth::Good,
        inserted_at: now,
        updated_at: now,
    })
}

fn read_sysfs_attr(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_mounts() -> Vec<(String, String)> {
    let content = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    content.lines().filter_map(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0].starts_with("/dev/") {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }).collect()
}

fn check_system_disk(name: &str, mounts: &[(String, String)]) -> bool {
    if name == "sda" || name.starts_with("nvme0n1") || name.starts_with("mmcblk0") {
        return true;
    }
    let dev = format!("/dev/{}", name);
    mounts.iter().any(|(d, m)| {
        d.starts_with(&dev) && (m == "/" || m == "/boot" || m.starts_with("/boot/") || m.contains("docker"))
    })
}
