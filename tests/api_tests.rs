// Vigilant API integration tests
mod common;

use axum::body::Body;
use http::Request;
use serde_json::{Value, json};
use tower::ServiceExt;

use vigilant::db::DbPool;

use common::*;

// -- Helpers --

async fn get(router: &mut axum::Router, path: &str) -> (u16, Value) {
    let resp = router
        .as_service()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    let json: Value =
        serde_json::from_slice(&body).unwrap_or(json!({"raw": String::from_utf8_lossy(&body)}));
    (status, json)
}

async fn post(router: &mut axum::Router, path: &str, payload: &Value) -> (u16, Value) {
    let body_str = payload.to_string();
    let resp = router
        .as_service()
        .oneshot(
            Request::post(path)
                .header("Content-Type", "application/json")
                .body(Body::from(body_str))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    let json: Value =
        serde_json::from_slice(&body).unwrap_or(json!({"raw": String::from_utf8_lossy(&body)}));
    (status, json)
}

// -- Daily Uptime --

async fn insert_checks_at(pool: &DbPool, monitor_id: &str, entries: &[(&str, &str, Option<i64>)]) {
    for (timestamp, status, rt) in entries {
        sqlx::query(
            "INSERT INTO checks (monitor_id, status, response_time_ms, checked_at) VALUES (?, ?, ?, ?)"
        )
        .bind(monitor_id)
        .bind(status)
        .bind(rt)
        .bind(timestamp)
        .execute(pool.as_sqlite())
        .await
        .expect("insert check");
    }
}

#[tokio::test]
async fn daily_uptime_aggregates_correctly() {
    let (mut router, pool) = setup_test_app().await;
    let monitor_id = seed_monitor(&pool, "test-api", "healthy").await;

    // Day 1 (2026-08-06): 3 healthy
    // Day 2 (2026-08-07): 2 healthy, 1 dead
    let checks = vec![
        ("2026-08-06T10:00:00", "healthy", Some(50i64)),
        ("2026-08-06T14:00:00", "healthy", Some(60i64)),
        ("2026-08-06T18:00:00", "healthy", Some(55i64)),
        ("2026-08-07T08:00:00", "healthy", Some(50i64)),
        ("2026-08-07T12:00:00", "healthy", Some(200i64)),
        ("2026-08-07T16:00:00", "dead", None),
    ];
    insert_checks_at(&pool, &monitor_id, &checks).await;

    let (status, json) = get(
        &mut router,
        &format!("/api/monitors/{}/uptime/daily?days=90", monitor_id),
    )
    .await;
    assert_eq!(status, 200);
    let days = json.as_array().unwrap();
    assert_eq!(days.len(), 2);

    // Day 1: 3/3 healthy = 100%
    assert_eq!(days[0]["date"], "2026-08-06");
    assert_eq!(days[0]["uptime_percent"], 100.0);
    assert_eq!(days[0]["healthy"], 3);
    assert_eq!(days[0]["dead"], 0);

    // Day 2: 2/3 healthy = 66.67%
    assert_eq!(days[1]["date"], "2026-08-07");
    assert!((days[1]["uptime_percent"].as_f64().unwrap() - 66.67).abs() < 0.01);
    assert_eq!(days[1]["healthy"], 2);
    assert_eq!(days[1]["dead"], 1);
}

#[tokio::test]
async fn daily_uptime_empty_monitor() {
    let (mut router, pool) = setup_test_app().await;
    let monitor_id = seed_monitor(&pool, "empty", "healthy").await;

    let (status, json) = get(
        &mut router,
        &format!("/api/monitors/{}/uptime/daily?days=90", monitor_id),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn daily_uptime_days_param_respected() {
    let (mut router, pool) = setup_test_app().await;
    let monitor_id = seed_monitor(&pool, "partial", "healthy").await;

    // Checks spread over 5 distinct dates, but we only query 3 days
    let checks = vec![
        ("2026-08-04T12:00:00", "healthy", Some(50i64)),
        ("2026-08-05T12:00:00", "healthy", Some(50i64)),
        ("2026-08-06T12:00:00", "healthy", Some(50i64)),
        ("2026-08-07T12:00:00", "healthy", Some(50i64)),
        ("2026-08-08T12:00:00", "healthy", Some(50i64)),
    ];
    insert_checks_at(&pool, &monitor_id, &checks).await;

    let (status, json) = get(
        &mut router,
        &format!("/api/monitors/{}/uptime/daily?days=3", monitor_id),
    )
    .await;
    assert_eq!(status, 200);
    let days = json.as_array().unwrap();
    // 3-day window (relative to now) should contain fewer than all 5 dates
    assert!(
        days.len() < 5,
        "expected <5 days with days=3 filter, got {}",
        days.len()
    );
}

// -- Incidents --

#[tokio::test]
async fn incidents_list_ordered() {
    let (mut router, pool) = setup_test_app().await;
    let mid = seed_monitor(&pool, "incident-svc", "healthy").await;

    seed_incident(&pool, &mid, 48, true).await; // 48h ago, resolved
    seed_incident(&pool, &mid, 10, false).await; // 10h ago, open
    seed_incident(&pool, &mid, 72, true).await; // 72h ago, resolved

    let (status, json) = get(&mut router, "/api/incidents?limit=10").await;
    assert_eq!(status, 200);
    let list = json.as_array().unwrap();
    assert_eq!(list.len(), 3);

    // Ordered by started_at DESC: newest first (10h ago)
    assert!(list[0]["started_at"].as_str().unwrap() > list[1]["started_at"].as_str().unwrap());
    assert!(list[1]["started_at"].as_str().unwrap() > list[2]["started_at"].as_str().unwrap());
}

#[tokio::test]
async fn incidents_list_respects_limit() {
    let (mut router, pool) = setup_test_app().await;
    let mid = seed_monitor(&pool, "limited-svc", "healthy").await;

    for i in 1..=5 {
        seed_incident(&pool, &mid, i * 10, i % 2 == 0).await;
    }

    let (status, json) = get(&mut router, "/api/incidents?limit=2").await;
    assert_eq!(status, 200);
    assert_eq!(json.as_array().unwrap().len(), 2);
}

// -- Subscribe --

#[tokio::test]
async fn subscribe_valid_email() {
    let (mut router, _pool) = setup_test_app().await;

    let (status, json) = post(
        &mut router,
        "/api/subscribe",
        &json!({"email": "user@example.com"}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["ok"], true);
    assert!(json["message"].as_str().unwrap().contains("Subscribed"));
}

#[tokio::test]
async fn subscribe_invalid_email() {
    let (mut router, _pool) = setup_test_app().await;

    let (status, json) = post(
        &mut router,
        "/api/subscribe",
        &json!({"email": "notanemail"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn subscribe_duplicate() {
    let (mut router, _pool) = setup_test_app().await;

    post(
        &mut router,
        "/api/subscribe",
        &json!({"email": "dup@test.com"}),
    )
    .await;
    let (status, json) = post(
        &mut router,
        "/api/subscribe",
        &json!({"email": "dup@test.com"}),
    )
    .await;

    assert_eq!(status, 200);
    assert!(
        json["message"]
            .as_str()
            .unwrap()
            .contains("Already subscribed")
    );
}

// -- Atom Feed --

#[tokio::test]
async fn atom_feed_empty() {
    let (mut router, _pool) = setup_test_app().await;

    let resp = router
        .as_service()
        .oneshot(Request::get("/api/feed/atom").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let ct = resp
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("atom"), "expected atom content-type, got: {ct}");

    let body = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&body);
    assert!(xml.contains("<feed"), "expected <feed> root element");
    assert!(
        !xml.contains("<entry>"),
        "expected no entries in empty feed"
    );
}

#[tokio::test]
async fn atom_feed_with_incidents() {
    let (mut router, pool) = setup_test_app().await;
    let mid = seed_monitor(&pool, "atom-svc", "healthy").await;

    seed_incident(&pool, &mid, 5, true).await;
    seed_incident(&pool, &mid, 24, true).await;

    let resp = router
        .as_service()
        .oneshot(
            Request::get("/api/feed/atom?limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    let xml = String::from_utf8_lossy(&body);

    assert!(xml.contains("<feed"), "expected <feed> root");
    // Two entries
    assert_eq!(xml.matches("<entry>").count(), 2);
    // Contains monitor name
    assert!(
        xml.contains("atom-svc"),
        "expected monitor name in feed, got: {xml}"
    );
    // Contains [Resolved]
    assert!(xml.contains("[Resolved]"), "expected [Resolved] in title");
}

#[tokio::test]
async fn atom_feed_content_type() {
    let (mut router, _pool) = setup_test_app().await;

    let resp = router
        .as_service()
        .oneshot(Request::get("/api/feed/atom").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let ct = resp
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.starts_with("application/atom+xml"), "got: {ct}");
}

// -- Health --

#[tokio::test]
async fn health_live_ok() {
    let (mut router, _pool) = setup_test_app().await;

    let (status, json) = get(&mut router, "/health/live").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn health_ready_ok() {
    let (mut router, _pool) = setup_test_app().await;

    let (status, json) = get(&mut router, "/health/ready").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "ok");
}

// -- Status --

#[tokio::test]
async fn status_overall_healthy() {
    let (mut router, pool) = setup_test_app().await;
    seed_monitor(&pool, "healthy-1", "healthy").await;
    seed_monitor(&pool, "healthy-2", "healthy").await;

    let (status, json) = get(&mut router, "/api/status").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["monitors"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn status_overall_dead() {
    let (mut router, pool) = setup_test_app().await;
    seed_monitor(&pool, "alive", "healthy").await;
    seed_monitor(&pool, "dead-one", "dead").await;

    let (status, json) = get(&mut router, "/api/status").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "dead");
}

#[tokio::test]
async fn status_overall_sick() {
    let (mut router, pool) = setup_test_app().await;
    seed_monitor(&pool, "ok", "healthy").await;
    seed_monitor(&pool, "degraded", "sick").await;

    let (status, json) = get(&mut router, "/api/status").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "sick");
}
