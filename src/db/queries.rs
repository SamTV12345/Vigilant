// Vigilant
// Database query functions
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::models::*;

// -- Monitors --

pub async fn list_monitors(pool: &SqlitePool) -> Result<Vec<Monitor>, sqlx::Error> {
    sqlx::query_as::<_, Monitor>("SELECT * FROM monitors ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_monitor(pool: &SqlitePool, id: &str) -> Result<Option<Monitor>, sqlx::Error> {
    sqlx::query_as::<_, Monitor>("SELECT * FROM monitors WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create_monitor(
    pool: &SqlitePool,
    input: &CreateMonitor,
) -> Result<Monitor, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let headers = input.headers.as_deref().unwrap_or("{}");
    sqlx::query(
        "INSERT INTO monitors (id, name, type, url, interval_secs, timeout_secs, method, headers, body, script)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.type_)
    .bind(&input.url)
    .bind(input.interval_secs)
    .bind(input.timeout_secs)
    .bind(input.method.as_deref().unwrap_or("GET"))
    .bind(headers)
    .bind(&input.body)
    .bind(&input.script)
    .execute(pool)
    .await?;
    get_monitor(pool, &id).await.map(|m| m.unwrap())
}

pub async fn update_monitor(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateMonitor,
) -> Result<Option<Monitor>, sqlx::Error> {
    let existing = get_monitor(pool, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let e = existing.unwrap();

    sqlx::query(
        "UPDATE monitors SET name=?, url=?, interval_secs=?, timeout_secs=?, method=?, headers=?, body=?, script=?, active=?, updated_at=datetime('now') WHERE id=?"
    )
    .bind(input.name.as_deref().unwrap_or(&e.name))
    .bind(input.url.as_deref().unwrap_or(&e.url))
    .bind(input.interval_secs.unwrap_or(e.interval_secs))
    .bind(input.timeout_secs.unwrap_or(e.timeout_secs))
    .bind(input.method.as_deref().unwrap_or(e.method.as_deref().unwrap_or("GET")))
    .bind(input.headers.as_deref().unwrap_or(e.headers.as_deref().unwrap_or("{}")))
    .bind(input.body.as_deref().or(e.body.as_deref()))
    .bind(input.script.as_deref().or(e.script.as_deref()))
    .bind(input.active.unwrap_or(e.active))
    .bind(id)
    .execute(pool)
    .await?;
    get_monitor(pool, id).await
}

pub async fn delete_monitor(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM monitors WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_monitor_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE monitors SET current_status = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// -- Checks --

pub async fn insert_check(
    pool: &SqlitePool,
    monitor_id: &str,
    status: &str,
    response_time_ms: Option<i64>,
    status_code: Option<i64>,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO checks (monitor_id, status, response_time_ms, status_code, error) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(monitor_id)
    .bind(status)
    .bind(response_time_ms)
    .bind(status_code)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_checks(
    pool: &SqlitePool,
    monitor_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Check>, sqlx::Error> {
    sqlx::query_as::<_, Check>(
        "SELECT * FROM checks WHERE monitor_id = ? ORDER BY checked_at DESC LIMIT ? OFFSET ?",
    )
    .bind(monitor_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

#[derive(Debug, serde::Serialize)]
pub struct DailyUptime {
    pub date: String,
    pub uptime_percent: f64,
    pub healthy: i64,
    pub sick: i64,
    pub dead: i64,
}

pub async fn get_daily_uptime(
    pool: &SqlitePool,
    monitor_id: &str,
    days: i64,
) -> Result<Vec<DailyUptime>, sqlx::Error> {
    let since = format!("-{} days", days);
    let rows = sqlx::query(
        "SELECT date(checked_at) as day, COUNT(*) as total,
                SUM(CASE WHEN status = 'healthy' THEN 1 ELSE 0 END) as healthy,
                SUM(CASE WHEN status = 'sick' THEN 1 ELSE 0 END) as sick,
                SUM(CASE WHEN status = 'dead' THEN 1 ELSE 0 END) as dead
         FROM checks WHERE monitor_id = ? AND checked_at >= datetime('now', ?)
         GROUP BY day ORDER BY day ASC",
    )
    .bind(monitor_id)
    .bind(&since)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| {
            let healthy: i64 = r.get(2);
            let sick: i64 = r.get(3);
            let dead: i64 = r.get(4);
            let total = healthy + sick + dead;
            DailyUptime {
                date: r.get(0),
                uptime_percent: if total > 0 {
                    (healthy as f64 / total as f64) * 100.0
                } else {
                    100.0
                },
                healthy,
                sick,
                dead,
            }
        })
        .collect())
}

pub async fn get_uptime(
    pool: &SqlitePool,
    monitor_id: &str,
    period_hours: i64,
) -> Result<UptimeResponse, sqlx::Error> {
    let since = format!("-{} hours", period_hours);
    let checks: Vec<Check> = sqlx::query_as(
        "SELECT * FROM checks WHERE monitor_id = ? AND checked_at >= datetime('now', ?) ORDER BY checked_at DESC"
    )
    .bind(monitor_id)
    .bind(&since)
    .fetch_all(pool)
    .await?;

    let total = checks.len() as i64;
    let healthy = checks.iter().filter(|c| c.status == "healthy").count() as i64;
    let sick = checks.iter().filter(|c| c.status == "sick").count() as i64;
    let dead = checks.iter().filter(|c| c.status == "dead").count() as i64;
    let uptime = if total > 0 {
        (healthy as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    Ok(UptimeResponse {
        monitor_id: monitor_id.to_string(),
        period_hours,
        uptime_percent: (uptime * 100.0).round() / 100.0,
        total_checks: total,
        healthy_checks: healthy,
        sick_checks: sick,
        dead_checks: dead,
    })
}

// -- Notifications --

pub async fn list_notifications(pool: &SqlitePool) -> Result<Vec<Notification>, sqlx::Error> {
    sqlx::query_as::<_, Notification>("SELECT * FROM notifications ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn create_notification(
    pool: &SqlitePool,
    input: &CreateNotification,
) -> Result<Notification, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let config = input.config.to_string();
    sqlx::query(
        "INSERT INTO notifications (id, name, type, config, reminders_only) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.type_)
    .bind(&config)
    .bind(input.reminders_only)
    .execute(pool)
    .await?;
    Ok(Notification {
        id,
        name: input.name.clone(),
        type_: input.type_.clone(),
        config,
        reminders_only: input.reminders_only,
        active: true,
        created_at: String::new(),
        updated_at: String::new(),
    })
}

pub async fn update_notification(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateNotification,
) -> Result<Option<Notification>, sqlx::Error> {
    let existing = sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    if existing.is_none() {
        return Ok(None);
    }
    let e = existing.unwrap();

    let config = input
        .config
        .as_ref()
        .map(|c| c.to_string())
        .unwrap_or(e.config);
    sqlx::query(
        "UPDATE notifications SET name=?, config=?, reminders_only=?, active=?, updated_at=datetime('now') WHERE id=?"
    )
    .bind(input.name.as_deref().unwrap_or(&e.name))
    .bind(&config)
    .bind(input.reminders_only.unwrap_or(e.reminders_only))
    .bind(input.active.unwrap_or(e.active))
    .bind(id)
    .execute(pool)
    .await?;
    sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .map(Some)
}

pub async fn delete_notification(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// -- Settings --

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
}

pub async fn list_settings(pool: &SqlitePool) -> Result<Vec<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT * FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
}

pub async fn upsert_setting(pool: &SqlitePool, input: &UpsertSetting) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(&input.key)
    .bind(&input.value)
    .execute(pool)
    .await?;
    Ok(())
}

// -- Users --

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn list_users(pool: &SqlitePool) -> Result<Vec<UserInfo>, sqlx::Error> {
    sqlx::query_as::<_, UserInfo>(
        "SELECT id, username, must_change_password, created_at FROM users ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> Result<UserInfo, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, must_change_password) VALUES (?, ?, ?, 1)",
    )
    .bind(&id)
    .bind(username)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(UserInfo {
        id,
        username: username.to_string(),
        must_change_password: 1,
        created_at: String::new(),
    })
}

pub async fn update_user_password(
    pool: &SqlitePool,
    id: &str,
    new_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ?, must_change_password = 0 WHERE id = ?")
        .bind(new_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// -- Announcements --

pub async fn list_announcements(pool: &SqlitePool) -> Result<Vec<Announcement>, sqlx::Error> {
    sqlx::query_as::<_, Announcement>("SELECT * FROM announcements ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn create_announcement(
    pool: &SqlitePool,
    input: &CreateAnnouncement,
) -> Result<Announcement, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO announcements (id, title, text) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&input.title)
        .bind(&input.text)
        .execute(pool)
        .await?;
    Ok(Announcement {
        id,
        title: input.title.clone(),
        text: input.text.clone(),
        created_at: String::new(),
    })
}

// -- Incidents --

pub async fn list_incidents(pool: &SqlitePool, limit: i64) -> Result<Vec<Incident>, sqlx::Error> {
    sqlx::query_as::<_, Incident>("SELECT * FROM incidents ORDER BY started_at DESC LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn create_incident(pool: &SqlitePool, monitor_id: &str) -> Result<Incident, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO incidents (id, monitor_id) VALUES (?, ?)")
        .bind(&id)
        .bind(monitor_id)
        .execute(pool)
        .await?;
    sqlx::query_as::<_, Incident>("SELECT * FROM incidents WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn resolve_incident(pool: &SqlitePool, monitor_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE incidents SET resolved_at = datetime('now'), status = 'resolved' WHERE monitor_id = ? AND resolved_at IS NULL"
    )
    .bind(monitor_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_announcement(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM announcements WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// -- Subscribers --

pub async fn add_subscriber(pool: &SqlitePool, email: &str) -> Result<bool, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let result = sqlx::query("INSERT OR IGNORE INTO subscribers (id, email) VALUES (?, ?)")
        .bind(&id)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// -- Feed data --

pub async fn get_recent_incidents_with_monitors(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<(Incident, String)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT i.id, i.monitor_id, i.started_at, i.resolved_at, i.status, COALESCE(m.name, i.monitor_id) as monitor_name
         FROM incidents i
         LEFT JOIN monitors m ON m.id = i.monitor_id
         ORDER BY i.started_at DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| {
            (
                Incident {
                    id: r.get(0),
                    monitor_id: r.get(1),
                    started_at: r.get(2),
                    resolved_at: r.get(3),
                    status: r.get(4),
                },
                r.get(5),
            )
        })
        .collect())
}
