use axum::{Json, extract::{State, Path}, response::IntoResponse};
use uuid::Uuid;

use crate::AppState;
use crate::domain::flash::*;

pub async fn start(
    State(state): State<AppState>,
    Json(request): Json<FlashRequest>,
) -> impl IntoResponse {
    match state.job_queue.enqueue(request).await {
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
