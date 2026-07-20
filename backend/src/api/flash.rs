use axum::{Json, extract::{State, Path}, response::IntoResponse};
use uuid::Uuid;

use crate::AppState;
use crate::domain::flash::*;
use crate::service::iso::IsoService;
use crate::service::usb::UsbService;

pub async fn start(
    State(state): State<AppState>,
    Json(request): Json<FlashRequest>,
) -> impl IntoResponse {
    let iso_images = IsoService::scan_directory(&state.config.storage.iso_path).await;
    let iso = match iso_images.iter().find(|img| img.id == request.iso_id) {
        Some(img) => img,
        None => return Json(serde_json::json!({"error": "ISO image not found"})).into_response(),
    };

    let usb_devices = UsbService::scan_devices().await;
    let usb = match usb_devices.iter().find(|dev| dev.id == request.usb_id) {
        Some(dev) => dev,
        None => return Json(serde_json::json!({"error": "USB device not found"})).into_response(),
    };

    match state.job_queue.enqueue(request, &iso.file_path, &usb.device_path, iso.file_size_bytes).await {
        Ok(job) => Json(serde_json::json!(job)).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.job_queue.status(id).await {
        Some(job) => Json(serde_json::json!(job)).into_response(),
        None => Json(serde_json::json!({"error": "not found"})).into_response(),
    }
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.job_queue.cancel(id).await {
        Ok(()) => Json(serde_json::json!({"status": "cancelled"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}
