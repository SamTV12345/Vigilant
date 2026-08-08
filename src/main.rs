// Vigilant
// Microservices Status Page — Uptime Kuma style

#[macro_use]
extern crate log;

use std::sync::Arc;

use axum::Router;
use socketioxide::SocketIo;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use vigilant::AppState;
use vigilant::config::AppConfig;

#[tokio::main]
async fn main() {
    let app_config = Arc::new(AppConfig::load());

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("vigilant=debug,info"),
    )
    .init();

    info!("starting vigilant on {}", app_config.listen_addr);

    let pool = vigilant::db::init_pool(&app_config.database_url)
        .await
        .expect("failed to initialize database");

    // Socket.IO layer — emit events from API handlers via state.io
    let (io_layer, io) = SocketIo::new_layer();

    // Register default namespace so emit() works
    io.ns(
        "/",
        |_socket: socketioxide::extract::SocketRef| async move {},
    );

    // Heartbeat every 10s so frontend knows the connection is alive
    let hb_io = io.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            hb_io
                .emit(
                    "heartbeat",
                    &serde_json::json!({"time": chrono::Utc::now().to_rfc3339()}),
                )
                .ok();
        }
    });

    // Probe engine — polls monitors from DB and writes check results
    let probe_pool = pool.clone();
    let notifier = Arc::new(Mutex::new(vigilant::notifier::NotifierState::new(
        pool.clone(),
    )));
    tokio::spawn(async move { vigilant::prober::start(probe_pool, notifier).await });

    let state = AppState {
        db: pool,
        config: app_config.clone(),
        io: io.clone(),
    };

    let api_routes = vigilant::api::build_router(state.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let assets_dir = ServeDir::new(&app_config.assets_path);
    let spa_fallback = ServeFile::new(format!("{}/index.html", app_config.assets_path));
    let app = Router::new()
        .merge(api_routes)
        .layer(io_layer)
        .layer(cors)
        .fallback_service(assets_dir.fallback(spa_fallback));

    let listener = tokio::net::TcpListener::bind(&app_config.listen_addr)
        .await
        .expect("failed to bind");

    info!("listening on {}", app_config.listen_addr);

    axum::serve(listener, app).await.unwrap();
}
