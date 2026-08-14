// Vigilant
// Health endpoints — liveness and readiness probes (Kubernetes-style)
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

/// Liveness — the process is up. Always 200 while the server is serving.
pub async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response()
}

/// Readiness — the app can serve traffic. Fails (503) when the database is
/// unreachable, so orchestrators stop routing traffic until it recovers.
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.ping().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "unavailable", "error": e.to_string() })),
        )
            .into_response(),
    }
}
