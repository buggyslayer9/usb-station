use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::IntoResponse,
    extract::State,
};
use futures::StreamExt;
use tracing::info;
use serde_json::json;

use crate::AppState;

pub async fn handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket connected");

    // Send initial status
    let msg = json!({
        "type": "connected",
        "data": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });

    if socket.send(Message::Text(msg.to_string())).await.is_err() {
        return;
    }

    // Listen for messages and send periodic updates
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        // Handle incoming messages (subscribe to specific events, etc.)
        info!("WS message: {}", msg);

        let response = json!({
            "type": "pong",
            "data": { "echo": msg }
        });

        if socket.send(Message::Text(response.to_string())).await.is_err() {
            break;
        }
    }

    info!("WebSocket disconnected");
}
