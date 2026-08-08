// Vigilant
// Public status endpoint
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::AppState;
use crate::db::queries;

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let monitors = match queries::list_monitors(&state.db).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let all_healthy = monitors
        .iter()
        .all(|m| m.current_status == "healthy" || m.current_status == "partial");
    let any_dead = monitors.iter().any(|m| m.current_status == "dead");
    let any_sick = monitors.iter().any(|m| m.current_status == "sick");

    let overall = if any_dead {
        "dead"
    } else if any_sick {
        "sick"
    } else if all_healthy {
        "healthy"
    } else {
        "partial"
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": overall,
            "monitors": monitors.iter().map(|m| serde_json::json!({
                "id": m.id,
                "name": m.name,
                "type": m.type_,
                "url": m.url,
                "status": m.current_status,
                "active": m.active,
            })).collect::<Vec<_>>(),
        })),
    )
        .into_response()
}
