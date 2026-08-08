// Vigilant test helpers
use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;
use vigilant::AppState;
use vigilant::config::AppConfig;

/// Build a test app with in-memory SQLite. Returns (Router, SqlitePool).
pub async fn setup_test_app() -> (Router, SqlitePool) {
    // In-memory SQLite with shared cache so all pool connections see the same DB
    let pool = vigilant::db::init_pool("sqlite::memory:?cache=shared")
        .await
        .expect("failed to create test pool");

    let config = Arc::new(AppConfig {
        database_url: "sqlite::memory:?cache=shared".into(),
        jwt_secret: "test-secret".into(),
        listen_addr: "0.0.0.0:0".into(),
        assets_path: "./res/assets/".into(),
    });

    // Create a dummy SocketIo — required by AppState, not used by API tests
    let (_, io) = socketioxide::SocketIo::new_layer();
    io.ns("/", |_s: socketioxide::extract::SocketRef| async move {});

    let state = AppState {
        db: pool.clone(),
        config,
        io,
    };
    let router = vigilant::api::build_router(state);

    (router, pool)
}

/// Insert a monitor and return its ID.
pub async fn seed_monitor(pool: &SqlitePool, name: &str, status: &str) -> String {
    use vigilant::db::models::CreateMonitor;
    let m = vigilant::db::queries::create_monitor(
        pool,
        &CreateMonitor {
            name: name.into(),
            type_: "http".into(),
            url: format!("https://{}.test", name),
            interval_secs: 60,
            timeout_secs: 10,
            method: Some("GET".into()),
            headers: None,
            body: None,
            script: None,
        },
    )
    .await
    .expect("create monitor");
    let id = m.id;

    // Update to desired status
    let _ = sqlx::query("UPDATE monitors SET current_status = ? WHERE id = ?")
        .bind(status)
        .bind(&id)
        .execute(pool)
        .await;

    id
}

/// Insert an incident for a monitor.
pub async fn seed_incident(pool: &SqlitePool, monitor_id: &str, hours_ago: i64, resolved: bool) {
    let id = uuid::Uuid::new_v4().to_string();
    let since = format!("-{} hours", hours_ago);
    if resolved {
        sqlx::query(
            "INSERT INTO incidents (id, monitor_id, started_at, resolved_at, status)
             VALUES (?, ?, datetime('now', ?), datetime('now', ?, '+1 hour'), 'resolved')",
        )
        .bind(&id)
        .bind(monitor_id)
        .bind(&since)
        .bind(&since)
        .execute(pool)
        .await
        .expect("insert resolved incident");
    } else {
        sqlx::query(
            "INSERT INTO incidents (id, monitor_id, started_at, status)
             VALUES (?, ?, datetime('now', ?), 'investigating')",
        )
        .bind(&id)
        .bind(monitor_id)
        .bind(&since)
        .execute(pool)
        .await
        .expect("insert open incident");
    }
}
