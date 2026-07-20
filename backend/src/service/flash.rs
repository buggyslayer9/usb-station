use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use tracing::{info, error, warn};
use regex::Regex;

use crate::domain::flash::*;

static DD_PROGRESS_RE: OnceLock<Regex> = OnceLock::new();
fn dd_progress_re() -> &'static Regex {
    DD_PROGRESS_RE.get_or_init(|| {
        Regex::new(r"(\d+)\s+bytes.*?copied.*?([\d.]+)\s+s.*?([\d.]+)\s+(MB/s|GB/s)").unwrap()
    })
}

#[derive(Debug, Clone)]
pub enum FlashCommand {
    Execute {
        job_id: Uuid,
        iso_path: String,
        device_path: String,
    },
    Cancel(Uuid),
}

#[derive(Clone)]
pub struct FlashJobQueue {
    jobs: Arc<RwLock<HashMap<Uuid, FlashJob>>>,
    progress_tx: broadcast::Sender<FlashJobProgress>,
    cmd_tx: broadcast::Sender<FlashCommand>,
    cancel_tokens: Arc<RwLock<HashMap<Uuid, CancellationToken>>>,
}

impl FlashJobQueue {
    pub fn new(_max_concurrent: u32) -> (Self, broadcast::Receiver<FlashCommand>) {
        let (progress_tx, _) = broadcast::channel(256);
        let (cmd_tx, cmd_rx) = broadcast::channel(256);
        let queue = Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            progress_tx,
            cmd_tx,
            cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
        };
        (queue, cmd_rx)
    }

    pub async fn enqueue(
        &self,
        request: FlashRequest,
        iso_path: &str,
        device_path: &str,
        total_bytes: u64,
    ) -> Result<FlashJob, String> {
        let job = FlashJob {
            id: Uuid::new_v4(),
            iso_id: request.iso_id,
            usb_id: request.usb_id,
            batch_id: None,
            status: FlashStatus::Queued,
            progress_percent: 0.0,
            speed_bytes_per_sec: 0,
            bytes_written: 0,
            total_bytes,
            eta_seconds: None,
            verify: request.verify.unwrap_or(true),
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.jobs.write().await.insert(job.id, job.clone());

        let _ = self.cmd_tx.send(FlashCommand::Execute {
            job_id: job.id,
            iso_path: iso_path.to_string(),
            device_path: device_path.to_string(),
        });

        info!(job_id = %job.id, "Flash job enqueued");
        Ok(job)
    }

    pub async fn cancel(&self, job_id: Uuid) -> Result<(), String> {
        {
            let mut jobs = self.jobs.write().await;
            let job = jobs.get_mut(&job_id)
                .ok_or_else(|| format!("Job {} not found", job_id))?;

            if job.status == FlashStatus::Completed || job.status == FlashStatus::Failed {
                return Err(format!("Job {} already finished", job_id));
            }

            job.status = FlashStatus::Cancelled;
            job.updated_at = chrono::Utc::now();
        }

        {
            let tokens = self.cancel_tokens.read().await;
            if let Some(token) = tokens.get(&job_id) {
                token.cancel();
            }
        }

        let _ = self.cmd_tx.send(FlashCommand::Cancel(job_id));
        info!(job_id = %job_id, "Flash job cancelled");
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<FlashJobProgress> {
        self.progress_tx.subscribe()
    }

    pub async fn claim_job(&self, job_id: Uuid) -> bool {
        let mut jobs = self.jobs.write().await;
        match jobs.get_mut(&job_id) {
            Some(job) if job.status == FlashStatus::Queued => {
                job.status = FlashStatus::Flashing;
                job.started_at = Some(chrono::Utc::now());
                job.updated_at = chrono::Utc::now();
                true
            }
            _ => false,
        }
    }

    pub async fn status(&self, job_id: Uuid) -> Option<FlashJob> {
        self.jobs.read().await.get(&job_id).cloned()
    }

    pub async fn execute(&self, job_id: Uuid, iso_path: &str, device_path: &str) -> Result<(), String> {
        let cancel_token = {
            let token = CancellationToken::new();
            self.cancel_tokens.write().await.insert(job_id, token.clone());
            token
        };

        let engine = self.select_engine().await;
        info!(job_id = %job_id, engine = %engine, device = %device_path, "Starting flash");

        let result = match engine.as_str() {
            "dd" => self.flash_with_dd(job_id, iso_path, device_path, &cancel_token).await,
            "bmaptool" => self.flash_with_bmaptool(job_id, iso_path, device_path).await,
            _ => Err("No suitable flash engine found".into()),
        };

        self.cancel_tokens.write().await.remove(&job_id);

        let is_cancelled = {
            let jobs = self.jobs.read().await;
            if let Some(job) = jobs.get(&job_id) {
                job.status == FlashStatus::Cancelled
            } else {
                false
            }
        };

        if is_cancelled {
            return Err("Job was cancelled".into());
        }

        match &result {
            Ok(()) => {
                let needs_verify = {
                    let jobs = self.jobs.read().await;
                    jobs.get(&job_id).map(|j| j.verify).unwrap_or(false)
                };

                if needs_verify {
                    self.update_status(job_id, FlashStatus::Verifying).await;
                    match self.verify_write(iso_path, device_path, job_id).await {
                        Ok(()) => info!(job_id = %job_id, "Verification passed"),
                        Err(e) => {
                            self.update_status(job_id, FlashStatus::Failed).await;
                            let mut jobs = self.jobs.write().await;
                            if let Some(job) = jobs.get_mut(&job_id) {
                                job.error_message = Some(e.clone());
                                job.completed_at = Some(chrono::Utc::now());
                            }
                            return Err(e);
                        }
                    }
                }

                info!(job_id = %job_id, "Flash completed successfully");
                self.update_status(job_id, FlashStatus::Completed).await;
                let mut jobs = self.jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.progress_percent = 100.0;
                    job.completed_at = Some(chrono::Utc::now());
                }
            }
            Err(e) => {
                if !is_cancelled {
                    error!(job_id = %job_id, error = %e, "Flash failed");
                    self.update_status(job_id, FlashStatus::Failed).await;
                    let mut jobs = self.jobs.write().await;
                    if let Some(job) = jobs.get_mut(&job_id) {
                        job.error_message = Some(e.clone());
                        job.completed_at = Some(chrono::Utc::now());
                    }
                }
            }
        }

        result
    }

    async fn update_status(&self, job_id: Uuid, status: FlashStatus) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = status.clone();
            job.updated_at = chrono::Utc::now();
        }
        let _ = self.progress_tx.send(FlashJobProgress {
            job_id,
            status,
            progress_percent: 0.0,
            speed_bytes_per_sec: 0,
            bytes_written: 0,
            total_bytes: 0,
            eta_seconds: None,
        });
    }

    async fn select_engine(&self) -> String {
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

    async fn flash_with_dd(
        &self,
        job_id: Uuid,
        iso_path: &str,
        device_path: &str,
        cancel_token: &CancellationToken,
    ) -> Result<(), String> {
        let total_bytes = {
            let jobs = self.jobs.read().await;
            jobs.get(&job_id).map(|j| j.total_bytes).unwrap_or(0)
        };

        let mut child = Command::new("dd")
            .arg(format!("if={}", iso_path))
            .arg(format!("of={}", device_path))
            .arg("bs=4M")
            .arg("status=progress")
            .arg("oflag=direct")
            .arg("conv=fsync")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn dd: {}", e))?;

        let stderr = child.stderr.take().ok_or("Failed to capture dd stderr")?;
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line {
                        Ok(Some(text)) => {
                            if let Some(progress) = Self::parse_dd_progress(&text, total_bytes) {
                                let _ = self.progress_tx.send(FlashJobProgress {
                                    job_id,
                                    status: FlashStatus::Flashing,
                                    progress_percent: progress.progress_percent,
                                    speed_bytes_per_sec: progress.speed_bytes_per_sec,
                                    bytes_written: progress.bytes_written,
                                    total_bytes,
                                    eta_seconds: progress.eta_seconds,
                                });

                                let mut jobs = self.jobs.write().await;
                                if let Some(job) = jobs.get_mut(&job_id) {
                                    job.progress_percent = progress.progress_percent;
                                    job.speed_bytes_per_sec = progress.speed_bytes_per_sec;
                                    job.bytes_written = progress.bytes_written;
                                    job.eta_seconds = progress.eta_seconds;
                                    job.updated_at = chrono::Utc::now();
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            warn!(job_id = %job_id, error = %e, "Error reading dd output");
                            break;
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    let _ = child.kill().await;
                    warn!(job_id = %job_id, "DD process killed due to cancellation");
                    return Err("Cancelled".into());
                }
            }
        }

        let status = child.wait().await
            .map_err(|e| format!("Failed to wait for dd: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("dd failed with exit code: {:?}", status.code()))
        }
    }

    fn parse_dd_progress(line: &str, total_bytes: u64) -> Option<ParsedProgress> {
        let re = dd_progress_re();
        let caps = re.captures(line)?;

        let bytes_written: u64 = caps.get(1)?.as_str().parse().ok()?;
        let _secs: f64 = caps.get(2)?.as_str().parse().ok()?;
        let speed_val: f64 = caps.get(3)?.as_str().parse().ok()?;
        let speed_unit = caps.get(4)?.as_str();

        let speed_bytes_per_sec = match speed_unit {
            "GB/s" => (speed_val * 1_000_000_000.0) as u64,
            _ => (speed_val * 1_000_000.0) as u64,
        };

        let progress_percent = if total_bytes > 0 {
            (bytes_written as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let eta_seconds = if speed_bytes_per_sec > 0 && bytes_written < total_bytes {
            Some(((total_bytes - bytes_written) / speed_bytes_per_sec.max(1)) as u64)
        } else {
            None
        };

        Some(ParsedProgress {
            bytes_written,
            speed_bytes_per_sec,
            progress_percent,
            eta_seconds,
        })
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

    async fn verify_write(&self, iso_path: &str, device_path: &str, job_id: Uuid) -> Result<(), String> {
        info!(job_id = %job_id, "Starting SHA256 verification");

        let total_bytes = {
            let jobs = self.jobs.read().await;
            jobs.get(&job_id).map(|j| j.total_bytes).unwrap_or(0)
        };

        let iso_hash = Self::hash_file(iso_path, None).await?;
        let device_hash = Self::hash_file(device_path, Some(total_bytes)).await?;

        if iso_hash == device_hash {
            info!(job_id = %job_id, "SHA256 verification passed");
            Ok(())
        } else {
            error!(job_id = %job_id, iso_hash = %iso_hash, device_hash = %device_hash, "SHA256 mismatch");
            Err("SHA256 mismatch: written data does not match source ISO".into())
        }
    }

    async fn hash_file(path: &str, limit: Option<u64>) -> Result<String, String> {
        use sha2::{Sha256, Digest};
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(path).await
            .map_err(|e| format!("Failed to open {}: {}", path, e))?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 65536];
        let mut remaining = limit.unwrap_or(u64::MAX);

        loop {
            let max_read = buf.len().min(remaining as usize);
            if max_read == 0 {
                break;
            }
            let n = file.read(&mut buf[..max_read]).await
                .map_err(|e| format!("Failed to read {}: {}", path, e))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            remaining -= n as u64;
            if remaining == 0 {
                break;
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

struct ParsedProgress {
    bytes_written: u64,
    speed_bytes_per_sec: u64,
    progress_percent: f64,
    eta_seconds: Option<u64>,
}
