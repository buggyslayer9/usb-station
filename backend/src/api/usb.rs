use axum::{Json, extract::{State, Path}, response::IntoResponse};
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;
use crate::service::usb::UsbService;

#[derive(Serialize)]
pub struct UsbListResponse {
    pub devices: Vec<crate::domain::UsbDevice>,
}

pub async fn list(State(_state): State<AppState>) -> Json<UsbListResponse> {
    let devices = UsbService::scan_devices().await;
    Json(UsbListResponse { devices })
}

pub async fn eject(State(_state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    // In practice, look up the device by ID from state
    let result = UsbService::eject(&format!("/dev/sdb")).await;
    match result {
        Ok(()) => Json(serde_json::json!({"status": "ejected"})),
        Err(e) => Json(serde_json::json!({"error": e})),
    }
}
