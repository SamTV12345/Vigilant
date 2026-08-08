// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tera::Tera;
use time;
use uuid::Uuid;

use super::announcements::{
    Announcement, DATE_NOW_FORMATTER as ANNOUNCEMENTS_DATE_NOW_FORMATTER,
    STORE as ANNOUNCEMENTS_STORE,
};
use super::context::{IndexContext, INDEX_CONFIG, INDEX_ENVIRONMENT};
use super::payload::{
    ManagerAnnouncementInsertRequestPayload, ManagerAnnouncementInsertResponsePayload,
    ManagerAnnouncementsResponsePayload, ManagerProberAlertsIgnoredResolveRequestPayload,
    ManagerProberAlertsIgnoredResolveResponsePayload, ManagerProberAlertsResponsePayload,
    ManagerProberAlertsResponsePayloadEntry, ReporterRequestPayload, StatusReportResponsePayload,
};
use crate::prober::manager::{run_dispatch_plugins, STORE as PROBER_STORE};
use crate::prober::report::{
    handle_flush as handle_flush_report, handle_health as handle_health_report,
    handle_load as handle_load_report, HandleFlushError, HandleHealthError, HandleLoadError,
};
use crate::prober::status::Status;
use crate::APP_CONF;

pub type AppState = Arc<Tera>;

// -- Public routes --

pub async fn index(State(tera): State<AppState>) -> impl IntoResponse {
    let context = {
        IndexContext {
            states: &PROBER_STORE.read().unwrap().states,
            announcements: &ANNOUNCEMENTS_STORE.read().unwrap().announcements,
            environment: &*INDEX_ENVIRONMENT,
            config: &*INDEX_CONFIG,
        }
    };
    match tera.render("index.tera", &tera::Context::from_serialize(&context).unwrap()) {
        Ok(s) => Html(s).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template Error {:?}", e)).into_response(),
    }
}

pub async fn robots() -> impl IntoResponse {
    match tokio::fs::read(APP_CONF.assets.path.join("public").join("robots.txt")).await {
        Ok(contents) => Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(contents))
            .unwrap(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn status_text() -> impl IntoResponse {
    PROBER_STORE.read().unwrap().states.status.as_str().to_owned()
}

pub async fn status_report() -> impl IntoResponse {
    Json(StatusReportResponsePayload::build())
}

pub async fn badge(Path(kind): Path<String>) -> impl IntoResponse {
    let status = { PROBER_STORE.read().unwrap().states.status.as_str().to_owned() };
    let path = APP_CONF
        .assets
        .path
        .join("images")
        .join("badges")
        .join(format!("{}-{}-default.svg", kind, status));

    match tokio::fs::read(&path).await {
        Ok(contents) => Response::builder()
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(Body::from(contents))
            .unwrap(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// -- Reporter routes (authenticated externally) --

pub async fn reporter_report(
    Path((probe_id, node_id)): Path<(String, String)>,
    Json(data): Json<ReporterRequestPayload>,
) -> impl IntoResponse {
    debug!("reporter report: {}:{}", probe_id, node_id);

    if let Some(ref load) = data.load {
        match handle_load_report(&probe_id, &node_id, &data.replica, data.interval, load.cpu, load.ram) {
            Ok(forward) => {
                run_dispatch_plugins(&probe_id, &node_id, forward);
                StatusCode::OK
            }
            Err(HandleLoadError::InvalidLoad) => StatusCode::BAD_REQUEST,
            Err(HandleLoadError::WrongMode) => StatusCode::PRECONDITION_FAILED,
            Err(HandleLoadError::NotFound) => StatusCode::NOT_FOUND,
        }
    } else if let Some(ref health) = data.health {
        match handle_health_report(&probe_id, &node_id, &data.replica, data.interval, health) {
            Ok(_) => StatusCode::OK,
            Err(HandleHealthError::WrongMode) => StatusCode::PRECONDITION_FAILED,
            Err(HandleHealthError::NotFound) => StatusCode::NOT_FOUND,
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn reporter_flush(
    Path((probe_id, node_id, replica_id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    debug!("reporter flush: {}:{}:{}", probe_id, node_id, replica_id);

    match handle_flush_report(&probe_id, &node_id, &replica_id) {
        Ok(()) => StatusCode::OK,
        Err(HandleFlushError::WrongMode) => StatusCode::PRECONDITION_FAILED,
        Err(HandleFlushError::NotFound) => StatusCode::NOT_FOUND,
    }
}

// -- Manager routes (authenticated externally) --

pub async fn manager_announcements() -> impl IntoResponse {
    Json(
        ANNOUNCEMENTS_STORE
            .read()
            .unwrap()
            .announcements
            .iter()
            .map(|announcement| ManagerAnnouncementsResponsePayload {
                id: announcement.id.to_owned(),
                title: announcement.title.to_owned(),
            })
            .collect::<Vec<ManagerAnnouncementsResponsePayload>>(),
    )
}

pub async fn manager_announcement_insert(
    Json(data): Json<ManagerAnnouncementInsertRequestPayload>,
) -> impl IntoResponse {
    if data.title.len() > 0 && data.text.len() > 0 {
        let id = Uuid::new_v4().hyphenated().to_string();

        let mut store = ANNOUNCEMENTS_STORE.write().unwrap();
        store.announcements.push(Announcement {
            id: id.to_owned(),
            title: data.title.to_owned(),
            text: data.text.to_owned(),
            date: Some(
                time::OffsetDateTime::now_utc()
                    .format(&ANNOUNCEMENTS_DATE_NOW_FORMATTER)
                    .unwrap_or("?".to_string()),
            ),
        });

        Json(ManagerAnnouncementInsertResponsePayload { id }).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn manager_announcement_retract(Path(announcement_id): Path<String>) -> impl IntoResponse {
    let mut store = ANNOUNCEMENTS_STORE.write().unwrap();

    let announcement_index = store
        .announcements
        .iter()
        .position(|announcement| announcement.id == announcement_id);

    if let Some(announcement_index) = announcement_index {
        store.announcements.remove(announcement_index);
        StatusCode::OK.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn manager_prober_alerts() -> impl IntoResponse {
    let mut alerts = ManagerProberAlertsResponsePayload::default();
    let probes = &PROBER_STORE.read().unwrap().states.probes;

    for (probe_id, probe) in probes.iter() {
        for (node_id, node) in probe.nodes.iter() {
            for (replica_id, replica) in node.replicas.iter() {
                if replica.status == Status::Sick || replica.status == Status::Dead {
                    let alert_entry = ManagerProberAlertsResponsePayloadEntry {
                        probe: probe_id.to_owned(),
                        node: node_id.to_owned(),
                        replica: replica_id.to_owned(),
                    };

                    match replica.status {
                        Status::Sick => alerts.sick.push(alert_entry),
                        Status::Dead => alerts.dead.push(alert_entry),
                        _ => {}
                    }
                }
            }
        }
    }

    Json(alerts)
}

pub async fn manager_prober_alerts_ignored_resolve() -> impl IntoResponse {
    let states = &PROBER_STORE.read().unwrap().states;

    let reminders_seconds = states
        .notifier
        .reminder_ignore_until
        .and_then(|reminder_ignore_until| {
            reminder_ignore_until.duration_since(SystemTime::now()).ok()
        })
        .map(|reminder_ignore_duration_since| reminder_ignore_duration_since.as_secs() as u16);

    Json(ManagerProberAlertsIgnoredResolveResponsePayload {
        reminders_seconds,
    })
}

pub async fn manager_prober_alerts_ignored_update(
    Json(data): Json<ManagerProberAlertsIgnoredResolveRequestPayload>,
) -> impl IntoResponse {
    let mut store = PROBER_STORE.write().unwrap();

    store.states.notifier.reminder_ignore_until = data
        .reminders_seconds
        .map(|reminders_seconds| SystemTime::now() + Duration::from_secs(reminders_seconds as _));

    StatusCode::OK
}
