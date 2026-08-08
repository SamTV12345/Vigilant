// Vigilant
// Check history + uptime endpoints
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;

use crate::AppState;
use crate::db::queries;

#[derive(Deserialize)]
pub struct ChecksQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn checks(
    State(state): State<AppState>,
    Path(monitor_id): Path<String>,
    Query(q): Query<ChecksQuery>,
) -> impl IntoResponse {
    match queries::get_checks(&state.db, &monitor_id, q.limit, q.offset).await {
        Ok(checks) => (StatusCode::OK, Json(checks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct UptimeQuery {
    #[serde(default = "default_period")]
    pub period: i64,
}

fn default_period() -> i64 {
    24
}

pub async fn uptime(
    State(state): State<AppState>,
    Path(monitor_id): Path<String>,
    Query(q): Query<UptimeQuery>,
) -> impl IntoResponse {
    match queries::get_uptime(&state.db, &monitor_id, q.period).await {
        Ok(uptime) => (StatusCode::OK, Json(uptime)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct DailyQuery {
    #[serde(default = "default_days")]
    pub days: i64,
}

fn default_days() -> i64 {
    90
}

pub async fn daily_uptime(
    State(state): State<AppState>,
    Path(monitor_id): Path<String>,
    Query(q): Query<DailyQuery>,
) -> impl IntoResponse {
    match queries::get_daily_uptime(&state.db, &monitor_id, q.days).await {
        Ok(days) => (StatusCode::OK, Json(days)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
