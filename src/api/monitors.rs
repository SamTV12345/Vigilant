// Vigilant
// Monitor CRUD endpoints
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::{models::*, queries};

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match queries::list_monitors(&state.db).await {
        Ok(monitors) => (StatusCode::OK, Json(monitors)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match queries::get_monitor(&state.db, &id).await {
        Ok(Some(monitor)) => (StatusCode::OK, Json(monitor)).into_response(),
        Ok(None) => (
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

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateMonitor>,
) -> impl IntoResponse {
    match queries::create_monitor(&state.db, &input).await {
        Ok(monitor) => (StatusCode::CREATED, Json(monitor)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateMonitor>,
) -> impl IntoResponse {
    match queries::update_monitor(&state.db, &id, &input).await {
        Ok(Some(monitor)) => (StatusCode::OK, Json(monitor)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match queries::delete_monitor(&state.db, &id).await {
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
