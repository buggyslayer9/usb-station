use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::IntoResponse,
    extract::State,
};
use futures::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;
use uuid::Uuid;
use tracing::{info, warn};

use crate::AppState;
use crate::domain::flash::FlashJobProgress;
use crate::domain::batch::BatchProgress;
use crate::domain::usb::UsbDevice;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "usb_inserted")]
    UsbInserted(UsbDevice),
    #[serde(rename = "usb_removed")]
    UsbRemoved(Uuid),
    #[serde(rename = "flash_progress")]
    FlashProgress(FlashJobProgress),
    #[serde(rename = "flash_completed")]
    FlashCompleted(Uuid),
    #[serde(rename = "flash_failed")]
    FlashFailed(Uuid, String),
    #[serde(rename = "batch_progress")]
    BatchProgress(BatchProgress),
    #[serde(rename = "system_status")]
    SystemStatus { cpu: f64, memory: f64, uptime: u64 },
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    info!("WebSocket connected");

    let (mut sender, mut receiver) = socket.split();

    let connected = serde_json::json!({
        "type": "connected",
        "data": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });

    if sender.send(Message::Text(connected.to_string())).await.is_err() {
        return;
    }

    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        let json = serde_json::to_string(&msg).unwrap_or_default();
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket receiver lagged by {} messages", n);
                        continue;
                    }
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                            if val.get("type").and_then(|t| t.as_str()) == Some("ping") {
                                let pong = serde_json::json!({"type": "pong", "data": {}});
                                let _ = sender.send(Message::Text(pong.to_string())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Binary(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket disconnected");
}
