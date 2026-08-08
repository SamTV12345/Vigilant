// Vigilant
// Settings endpoints
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::{models::*, queries};

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match queries::list_settings(&state.db).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn upsert(
    State(state): State<AppState>,
    Json(input): Json<UpsertSetting>,
) -> impl IntoResponse {
    match queries::upsert_setting(&state.db, &input).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
