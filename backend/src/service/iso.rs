use std::path::Path;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use tracing::{info, warn};

use crate::domain::iso::*;

pub struct IsoService;

impl IsoService {
    /// Scan ISO directories and return discovered images.
    pub async fn scan_directory(path: &str) -> Vec<IsoImage> {
        let mut images = Vec::new();
        let dir = Path::new(path);

        if !dir.exists() {
            warn!(path = %path, "ISO directory does not exist");
            return images;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                warn!(path = %path, error = %e, "Failed to read ISO directory");
                return images;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            match ext.as_deref() {
                Some("iso") | Some("img") | Some("raw") | Some("bin") | Some("xz") | Some("gz") => {
                    let metadata = match std::fs::metadata(&path) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let (detected_os, detected_version) = Self::detect_os_from_filename(&filename);

                    images.push(IsoImage {
                        id: Uuid::new_v4(),
                        filename,
                        file_path: path.to_string_lossy().to_string(),
                        file_size_bytes: metadata.len(),
                        sha256: None,
                        md5: None,
                        detected_os,
                        detected_version,
                        boot_mode: Some(BootMode::Unknown),
                        category: Self::guess_category(ext.as_deref()),
                        tags: Vec::new(),
                        description: None,
                        is_favorite: false,
                        scanned_at: Some(chrono::Utc::now()),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    });
                }
                Some("zip") | Some("7z") => {
                    // Compressed archives — will be extracted on flash
                    // For now, just record them
                }
                _ => {}
            }
        }

        images
    }

    /// Compute SHA256 hash of an ISO file
    pub async fn compute_sha256(path: &str) -> Result<String, String> {
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        Ok(hash)
    }

    fn detect_os_from_filename(filename: &str) -> (Option<String>, Option<String>) {
        let lower = filename.to_lowercase();

        let known = [
            ("ubuntu", "Ubuntu"),
            ("debian", "Debian"),
            ("fedora", "Fedora"),
            ("arch", "Arch Linux"),
            ("centos", "CentOS"),
            ("rocky", "Rocky Linux"),
            ("almalinux", "AlmaLinux"),
            ("proxmox", "Proxmox VE"),
            ("truenas", "TrueNAS"),
            ("opnsense", "OPNsense"),
            ("pfsense", "pfSense"),
            ("esxi", "VMware ESXi"),
            ("windows", "Windows"),
            ("winpe", "Windows PE"),
            ("memtest", "Memtest86+"),
            ("clonezilla", "Clonezilla"),
            ("rescuezilla", "Rescuezilla"),
            ("systemrescue", "SystemRescue"),
            ("zimaos", "ZimaOS"),
            ("casaos", "CasaOS"),
            ("kali", "Kali Linux"),
            ("mint", "Linux Mint"),
            ("pop-os", "Pop!_OS"),
            ("manjaro", "Manjaro"),
        ];

        for (keyword, name) in known {
            if lower.contains(keyword) {
                // Try to extract version number
                let version = Self::extract_version(&lower);
                return (Some(name.into()), version);
            }
        }

        (None, None)
    }

    fn extract_version(filename: &str) -> Option<String> {
        use std::sync::OnceLock;
        static RE: OnceLock<regex::Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            regex::Regex::new(r"(\d+[._-]\d+(?:[._-]\d+)?)").unwrap()
        });
        re.find(filename)
            .map(|m| m.as_str().replace('_', ".").replace('-', "."))
    }

    fn guess_category(ext: Option<&str>) -> Option<String> {
        match ext {
            Some("iso") => Some("ISO Image".into()),
            Some("img") => Some("Disk Image".into()),
            Some("raw") => Some("Raw Image".into()),
            Some("bin") => Some("Binary Image".into()),
            Some("xz") | Some("gz") => Some("Compressed Image".into()),
            _ => None,
        }
    }
}
