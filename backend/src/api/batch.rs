use axum::{Json, extract::{State, Path}, response::IntoResponse};
use uuid::Uuid;

use crate::AppState;
use crate::domain::batch::*;

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<BatchCreateRequest>,
) -> impl IntoResponse {
    let iso_lookup = |id: Uuid| None; // TODO: query DB
    let usb_lookup = |id: Uuid| None;

    match state.batch_orchestrator.create_batch(request, iso_lookup, usb_lookup).await {
        Ok(batch) => Json(serde_json::json!(batch)).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.batch_orchestrator.batch_progress(id).await {
        Some(progress) => Json(serde_json::json!(progress)).into_response(),
        None => Json(serde_json::json!({"error": "not found"})).into_response(),
    }
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.batch_orchestrator.cancel_batch(id).await {
        Ok(()) => Json(serde_json::json!({"status": "cancelled"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}
