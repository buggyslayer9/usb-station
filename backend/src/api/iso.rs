use axum::{Json, extract::{State, Path}};
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;
use crate::service::iso::IsoService;

#[derive(Serialize)]
pub struct IsoListResponse {
    pub images: Vec<crate::domain::IsoImage>,
}

pub async fn list(State(state): State<AppState>) -> Json<IsoListResponse> {
    let images = IsoService::scan_directory(&state.config.storage.iso_path).await;
    Json(IsoListResponse { images })
}

pub async fn upload() -> Json<serde_json::Value> {
    Json(serde_json::json!({"message": "upload not yet implemented"}))
}

pub async fn delete(Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"deleted": id.to_string()}))
}
