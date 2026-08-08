// Vigilant
// API routes module
pub mod announcements;
pub mod auth;
pub mod checks;
pub mod feed;
pub mod incidents;
pub mod monitors;
pub mod notifications;
pub mod settings;
pub mod status;
pub mod subscribe;
pub mod users;

use axum::{
    Router,
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::AppState;
use auth::Claims;

// JWT middleware — validates Bearer token
async fn require_auth(
    state: axum::extract::State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = header {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        );
        if token_data.is_ok() {
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub fn build_router(state: AppState) -> Router {
    // Public routes
    let public = Router::new()
        .route("/status", get(status::status))
        .route("/monitors/{id}/checks", get(checks::checks))
        .route("/monitors/{id}/uptime", get(checks::uptime))
        .route("/monitors/{id}/uptime/daily", get(checks::daily_uptime))
        .route("/incidents", get(incidents::list))
        .route("/subscribe", post(subscribe::subscribe))
        .route("/feed/atom", get(feed::atom))
        .route("/announcements", get(announcements::list));

    // Auth route (no JWT needed)
    let auth = Router::new()
        .route("/login", post(auth::login))
        .route("/change-password", post(auth::change_password));

    // Admin routes (JWT protected)
    let admin = Router::new()
        .route("/monitors", get(monitors::list).post(monitors::create))
        .route(
            "/monitors/{id}",
            put(monitors::update).delete(monitors::delete),
        )
        .route(
            "/notifications",
            get(notifications::list).post(notifications::create),
        )
        .route(
            "/notifications/{id}",
            put(notifications::update).delete(notifications::delete),
        )
        .route("/settings", get(settings::list).put(settings::upsert))
        .route("/announcements", post(announcements::create))
        .route("/announcements/{id}", delete(announcements::delete))
        .route("/users", get(users::list_users).post(users::create_user))
        .route("/users/{id}", delete(users::delete_user))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .nest("/api", public)
        .nest("/api/auth", auth)
        .nest("/api/admin", admin)
        .with_state(state)
}
