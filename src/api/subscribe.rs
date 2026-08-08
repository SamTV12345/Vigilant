// Vigilant
// Public subscribe endpoint
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;

use crate::AppState;
use crate::db::queries;

#[derive(Deserialize)]
pub struct SubscribeInput {
    pub email: String,
}

pub async fn subscribe(
    State(state): State<AppState>,
    Json(input): Json<SubscribeInput>,
) -> impl IntoResponse {
    // Basic email validation
    let email = input.email.trim().to_lowercase();
    if !email.contains('@') || email.len() < 5 || email.len() > 254 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid email address"})),
        );
    }

    match queries::add_subscriber(&state.db, &email).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "message": "Subscribed successfully"})),
        ),
        Ok(false) => (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "message": "Already subscribed"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
