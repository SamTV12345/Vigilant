// Vigilant
// Atom feed endpoint
use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::AppState;
use crate::db::queries;

#[derive(Deserialize)]
pub struct FeedQuery {
    #[serde(default = "default_feed_limit")]
    pub limit: i64,
}

fn default_feed_limit() -> i64 {
    30
}

pub async fn atom(State(state): State<AppState>, Query(q): Query<FeedQuery>) -> impl IntoResponse {
    let incidents = match queries::get_recent_incidents_with_monitors(&state.db, q.limit).await {
        Ok(i) => i,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let updated = incidents
        .first()
        .map(|(i, _)| i.started_at.clone())
        .unwrap_or_else(|| String::from("2024-01-01T00:00:00Z"));

    let mut xml = String::from(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    xml.push_str(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#);
    xml.push_str("<title>Vigilant Status</title>");
    xml.push_str("<link href=\"/status\" rel=\"alternate\" />");
    xml.push_str("<link href=\"/api/feed/atom\" rel=\"self\" />");
    xml.push_str("<id>urn:vigilant:status</id>");
    xml.push_str(&format!("<updated>{}</updated>", escape_xml(&updated)));

    for (incident, monitor_name) in &incidents {
        let title = if incident.resolved_at.is_some() {
            format!("[Resolved] {} — incident", monitor_name)
        } else {
            format!("[{}] {} — incident", incident.status, monitor_name)
        };

        xml.push_str("<entry>");
        xml.push_str(&format!(
            "<id>urn:vigilant:incident:{}</id>",
            escape_xml(&incident.id)
        ));
        xml.push_str(&format!("<title>{}</title>", escape_xml(&title)));
        xml.push_str(&format!(
            "<content type=\"html\">&lt;p&gt;Monitor: {}&lt;/p&gt;&lt;p&gt;Status: {}&lt;/p&gt;&lt;p&gt;Started: {}&lt;/p&gt;{}</content>",
            escape_xml(monitor_name),
            escape_xml(&incident.status),
            escape_xml(&incident.started_at),
            incident.resolved_at.as_ref().map_or(String::new(), |r| format!("&lt;p&gt;Resolved: {}&lt;/p&gt;", escape_xml(r)))
        ));
        xml.push_str(&format!(
            "<published>{}</published>",
            escape_xml(&incident.started_at)
        ));
        if let Some(resolved) = &incident.resolved_at {
            xml.push_str(&format!("<updated>{}</updated>", escape_xml(resolved)));
        } else {
            xml.push_str(&format!(
                "<updated>{}</updated>",
                escape_xml(&incident.started_at)
            ));
        }
        xml.push_str("<link href=\"/status\" />");
        xml.push_str("</entry>");
    }

    xml.push_str("</feed>");

    use axum::response::IntoResponse;
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
