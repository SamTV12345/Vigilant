// Vigilant
// User management API endpoints
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::models::CreateUserRequest;

pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    match crate::db::queries::list_users(&state.db).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "database error"})),
        )
            .into_response(),
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if input.username.trim().is_empty() || input.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "username and password required"})),
        )
            .into_response();
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(input.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "hashing failed"})),
            )
                .into_response();
        }
    };

    match crate::db::queries::create_user(&state.db, &input.username, &hash).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE") {
                "username already exists"
            } else {
                "failed to create user"
            };
            (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": msg})),
            )
                .into_response()
        }
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Prevent deleting the last user
    let users = match crate::db::queries::list_users(&state.db).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "database error"})),
            )
                .into_response();
        }
    };

    if users.len() <= 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "cannot delete the last user"})),
        )
            .into_response();
    }

    match crate::db::queries::delete_user(&state.db, &id).await {
        Ok(true) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "user not found"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "database error"})),
        )
            .into_response(),
    }
}
