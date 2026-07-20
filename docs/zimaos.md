# USB Station on ZimaOS

## Quick Install (App Store)

Add this GitHub repo as a custom app store source in ZimaOS:

1. Open ZimaOS Dashboard → **App Store**
2. Click **Add Source**
3. Paste the ZIP URL:
   ```
   https://github.com/usb-station/appstore/archive/refs/heads/main.zip
   ```
4. Confirm and wait for the store to refresh
5. Search for **USB Station** and click **Install**

## Manual Install (via UI)

From the ZimaOS "Manual App Installation" form:

| Field | Value |
|-------|-------|
| Docker Image | `ghcr.io/usb-station/usb-station` |
| Tag | `0.1.0` |
| Title | `USB Station` |
| Web UI | `http://192.168.1.32:8081` |
| Network | `bridge` |
| Ports | `8081:8080` (Host:Container) |

### Volumes

| Host Path | Container Path |
|-----------|---------------|
| `/DATA/AppData/usb-station/config` | `/data` |
| `/DATA/AppData/usb-station/iso` | `/storage/iso` |
| `/DATA/AppData/usb-station/downloads` | `/storage/downloads` |
| `/DATA/AppData/usb-station/logs` | `/logs` |

### Devices

| Device |
|--------|
| `/dev/sda` |
| `/dev/sdb` |
| `/dev/sdc` |
| `/dev/sdd` |

### Capabilities

- `SYS_ADMIN`
- `SYS_RAWIO`
- `MKNOD`

### Environment

| Variable | Value |
|----------|-------|
| `FLASH_MAX_CONCURRENT` | `2` |
| `LOG_LEVEL` | `info` |
| `TZ` | `UTC` |

### Restart Policy

`unless-stopped`

## App Store Structure

```
usb-station/
├── store-config.json          # Store-level metadata
├── Apps/
│   └── usb-station/
│       ├── docker-compose.yml  # Compose file with x-casaos metadata
│       └── assets/
│           ├── icon.svg        # App icon (shown in dashboard)
│           └── thumbnail.svg   # Store thumbnail
└── ...
```

## Build & Publish

To build the combined Docker image for ZimaOS:

```bash
docker build -f docker/Dockerfile.combined -t usb-station:latest .
```

## Supported Architectures

- `amd64` — Intel/AMD 64-bit (including ZimaCube and standard x86 servers)
- `arm64` — Raspberry Pi 4/5, Rockchip, Apple Silicon

## Notes

- USB devices must be physically connected to the ZimaOS server
- USB flashing requires `--privileged` or `cap_add` + device passthrough
- ZimaOS dynamically allocates the web UI port via `WEBUI_PORT` (default: 8081)
- The app stores all data under `/DATA/AppData/usb-station/`
