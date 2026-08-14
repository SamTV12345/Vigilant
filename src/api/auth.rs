// Vigilant
// Auth endpoints + JWT middleware
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::models::{ChangePasswordRequest, LoginRequest, LoginResponse};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // username
    pub exp: usize,
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = crate::db::queries::get_user_by_username(&state.db, &input.username)
        .await
        .ok()
        .flatten();

    if let Some(user) = user {
        let parsed = PasswordHash::new(&user.password_hash).ok();
        if let Some(parsed_hash) = parsed
            && Argon2::default()
                .verify_password(input.password.as_bytes(), &parsed_hash)
                .is_ok()
        {
            let claims = Claims {
                sub: user.username.clone(),
                exp: chrono::Utc::now().timestamp() as usize + 86400 * 7, // 7 days
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            )
            .unwrap();
            return (
                StatusCode::OK,
                Json(LoginResponse {
                    token,
                    must_change_password: user.must_change_password != 0,
                }),
            )
                .into_response();
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "invalid credentials"})),
    )
        .into_response()
}

pub async fn change_password(
    State(state): State<AppState>,
    Json(input): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Look up user by username
    let user = match crate::db::queries::get_user_by_username(&state.db, &input.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "invalid username"})),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "database error"})),
            )
                .into_response();
        }
    };

    // Verify current password
    let parsed = PasswordHash::new(&user.password_hash).ok();
    let valid = parsed.is_some_and(|h| {
        Argon2::default()
            .verify_password(input.current_password.as_bytes(), &h)
            .is_ok()
    });

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "current password is incorrect"})),
        )
            .into_response();
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(input.new_password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "hashing failed"})),
            )
                .into_response();
        }
    };

    // Update password in database
    match crate::db::queries::update_user_password(&state.db, &user.id, &new_hash).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to update password"})),
        )
            .into_response(),
    }
}
