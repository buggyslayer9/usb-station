-- USB devices table
CREATE TABLE IF NOT EXISTS usb_devices (
    id TEXT PRIMARY KEY NOT NULL,
    device_path TEXT NOT NULL,
    vendor TEXT,
    model TEXT,
    serial TEXT,
    capacity_bytes INTEGER NOT NULL DEFAULT 0,
    filesystem TEXT,
    mount_point TEXT,
    is_mounted INTEGER NOT NULL DEFAULT 0,
    is_readonly INTEGER NOT NULL DEFAULT 0,
    is_system_disk INTEGER NOT NULL DEFAULT 0,
    health TEXT NOT NULL DEFAULT 'unknown',
    inserted_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- ISO images table
CREATE TABLE IF NOT EXISTS iso_images (
    id TEXT PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    sha256 TEXT,
    md5 TEXT,
    detected_os TEXT,
    detected_version TEXT,
    boot_mode TEXT,
    category TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    description TEXT,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    scanned_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Flash jobs table
CREATE TABLE IF NOT EXISTS flash_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    iso_id TEXT NOT NULL,
    usb_id TEXT NOT NULL,
    batch_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    progress_percent REAL NOT NULL DEFAULT 0.0,
    speed_bytes_per_sec INTEGER NOT NULL DEFAULT 0,
    bytes_written INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    eta_seconds INTEGER,
    verify INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (iso_id) REFERENCES iso_images(id),
    FOREIGN KEY (usb_id) REFERENCES usb_devices(id),
    FOREIGN KEY (batch_id) REFERENCES flash_batches(id)
);

-- Flash batches table
CREATE TABLE IF NOT EXISTS flash_batches (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    mode TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    total_jobs INTEGER NOT NULL DEFAULT 0,
    completed_jobs INTEGER NOT NULL DEFAULT 0,
    failed_jobs INTEGER NOT NULL DEFAULT 0,
    max_concurrent INTEGER NOT NULL DEFAULT 2,
    created_by TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    updated_at TEXT NOT NULL
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'operator',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- API tokens table
CREATE TABLE IF NOT EXISTS api_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    last_used_at TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    details TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Settings table (key-value store)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_flash_jobs_status ON flash_jobs(status);
CREATE INDEX IF NOT EXISTS idx_flash_jobs_batch ON flash_jobs(batch_id);
CREATE INDEX IF NOT EXISTS idx_flash_batches_status ON flash_batches(status);
CREATE INDEX IF NOT EXISTS idx_iso_images_category ON iso_images(category);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at);
