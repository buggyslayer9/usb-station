use axum::{Router, routing::{get, post, delete}, http::Method};
use tokio::sync::broadcast;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod api;
mod domain;
mod service;
mod infra;
mod worker;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("usb_station=debug".parse()?))
        .json()
        .init();

    let cfg = config::Settings::load()?;
    let db = infra::database::connect(&cfg.database.url).await?;
    infra::database::run_migrations(&db).await?;

    let frontend_path = std::path::Path::new(&cfg.server.frontend_dist);
    let has_frontend = frontend_path.join("index.html").exists();
    if has_frontend {
        tracing::info!("Serving frontend from: {}", cfg.server.frontend_dist);
    }

    let (ws_tx, _) = broadcast::channel::<api::ws::WsMessage>(256);

    let usb_monitor = infra::usb::UsbMonitor::new();
    let event_bus = infra::event_bus::InMemoryEventBus::new();
    let (job_queue, cmd_rx) = service::flash::FlashJobQueue::new(cfg.flash.max_concurrent);

    {
        let ws_tx = ws_tx.clone();
        let mut usb_rx = usb_monitor.subscribe();
        tokio::spawn(async move {
            use crate::domain::usb::UsbEvent;
            while let Ok(event) = usb_rx.recv().await {
                let ws_msg = match event {
                    UsbEvent::Inserted(device) => api::ws::WsMessage::UsbInserted(device),
                    UsbEvent::Removed(id) => api::ws::WsMessage::UsbRemoved(id),
                    UsbEvent::Changed(_) => continue,
                    UsbEvent::Error(_) => continue,
                };
                let _ = ws_tx.send(ws_msg);
            }
        });
    }

    let app_state = AppState {
        db,
        config: cfg.clone(),
        usb_monitor,
        job_queue,
        batch_orchestrator: service::batch::BatchOrchestrator::new(),
        event_bus,
        ws_tx,
    };

    worker::start_workers(app_state.clone(), cmd_rx).await;

    let app = Router::new()
        .route("/api/health", get(api::health::check))
        .route("/api/usb", get(api::usb::list))
        .route("/api/usb/:id/eject", post(api::usb::eject))
        .route("/api/iso", get(api::iso::list))
        .route("/api/iso", post(api::iso::upload))
        .route("/api/iso/:id", delete(api::iso::delete))
        .route("/api/flash", post(api::flash::start))
        .route("/api/flash/:id", get(api::flash::status))
        .route("/api/flash/:id/cancel", post(api::flash::cancel))
        .route("/api/batch", post(api::batch::create))
        .route("/api/batch/:id", get(api::batch::status))
        .route("/api/batch/:id/cancel", post(api::batch::cancel))
        .route("/api/ws", get(api::ws::handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(Any)
            .allow_headers(Any))
        .with_state(app_state);

    let app = if has_frontend {
        let serve_dir = ServeDir::new(&cfg.server.frontend_dist)
            .append_index_html_on_directories(true);
        app.fallback_service(serve_dir)
    } else {
        app
    };

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    tracing::info!("USB Station starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: config::Settings,
    pub usb_monitor: infra::usb::UsbMonitor,
    pub job_queue: service::flash::FlashJobQueue,
    pub batch_orchestrator: service::batch::BatchOrchestrator,
    pub event_bus: infra::event_bus::InMemoryEventBus,
    pub ws_tx: broadcast::Sender<api::ws::WsMessage>,
}
