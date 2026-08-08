// Vigilant
// Notification channel CRUD
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::{models::*, queries};

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    match queries::list_notifications(&state.db).await {
        Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNotification>,
) -> impl IntoResponse {
    match queries::create_notification(&state.db, &input).await {
        Ok(notification) => (StatusCode::CREATED, Json(notification)).into_response(),
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
    Json(input): Json<UpdateNotification>,
) -> impl IntoResponse {
    match queries::update_notification(&state.db, &id, &input).await {
        Ok(Some(notification)) => (StatusCode::OK, Json(notification)).into_response(),
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
    match queries::delete_notification(&state.db, &id).await {
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
