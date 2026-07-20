use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub flash: FlashConfig,
    pub storage: StorageConfig,
    pub watch: WatchConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_frontend_dist")]
    pub frontend_dist: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlashConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
    #[serde(default = "default_default_verify")]
    pub verify: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    #[serde(default = "default_iso_path")]
    pub iso_path: String,
    #[serde(default = "default_download_path")]
    pub download_path: String,
    #[serde(default = "default_log_path")]
    pub log_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WatchConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_watch_paths")]
    pub paths: Vec<String>,
}

fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 8080 }
fn default_log_level() -> String { "info".into() }
fn default_frontend_dist() -> String { "/usr/share/usb-station/frontend".into() }
fn default_db_url() -> String { "sqlite:///data/usb-station.db?mode=rwc".into() }
fn default_max_concurrent() -> u32 { 2 }
fn default_default_verify() -> bool { true }
fn default_iso_path() -> String { "/storage/iso".into() }
fn default_download_path() -> String { "/storage/downloads".into() }
fn default_log_path() -> String { "/logs".into() }
fn default_enabled() -> bool { true }
fn default_watch_paths() -> Vec<String> { vec!["/storage/iso".into()] }

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| default_host()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(default_port),
                log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| default_log_level()),
                frontend_dist: std::env::var("FRONTEND_DIST").unwrap_or_else(|_| default_frontend_dist()),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL").unwrap_or_else(|_| default_db_url()),
            },
            flash: FlashConfig {
                max_concurrent: std::env::var("FLASH_MAX_CONCURRENT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(default_max_concurrent),
                verify: std::env::var("FLASH_VERIFY")
                    .ok()
                    .map(|v| v == "true")
                    .unwrap_or_else(default_default_verify),
            },
            storage: StorageConfig {
                iso_path: std::env::var("STORAGE_ISO_PATH").unwrap_or_else(|_| default_iso_path()),
                download_path: std::env::var("STORAGE_DOWNLOAD_PATH").unwrap_or_else(|_| default_download_path()),
                log_path: std::env::var("STORAGE_LOG_PATH").unwrap_or_else(|_| default_log_path()),
            },
            watch: WatchConfig {
                enabled: std::env::var("WATCH_ENABLED")
                    .ok()
                    .map(|v| v == "true")
                    .unwrap_or_else(default_enabled),
                paths: std::env::var("WATCH_PATHS")
                    .ok()
                    .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_else(default_watch_paths),
            },
        })
    }
}
