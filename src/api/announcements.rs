// Vigilant
// Announcements endpoints
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::{models::*, queries};

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match queries::list_announcements(&state.db).await {
        Ok(announcements) => (StatusCode::OK, Json(announcements)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateAnnouncement>,
) -> impl IntoResponse {
    match queries::create_announcement(&state.db, &input).await {
        Ok(announcement) => (StatusCode::CREATED, Json(announcement)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match queries::delete_announcement(&state.db, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
