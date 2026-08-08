// Vigilant
// Public incidents endpoint
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;

use crate::AppState;
use crate::db::queries;

#[derive(Deserialize)]
pub struct IncidentsQuery {
    #[serde(default = "default_incident_limit")]
    pub limit: i64,
}

fn default_incident_limit() -> i64 {
    50
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<IncidentsQuery>,
) -> impl IntoResponse {
    match queries::list_incidents(&state.db, q.limit).await {
        Ok(incidents) => (StatusCode::OK, Json(incidents)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
