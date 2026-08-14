// Vigilant
// Database query functions
use std::borrow::Cow;

use chrono::{Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use super::DbPool;
use super::models::*;

// -- Dialect helpers --

/// Rewrite SQLite `?` placeholders to Postgres `$1`, `$2`, ... positionally.
/// Safe only for SQL without `?` inside string literals (true for all queries here).
fn pg_rebind(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut n = 0usize;
    for ch in sql.chars() {
        if ch == '?' {
            n += 1;
            out.push('$');
            out.push_str(&n.to_string());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Pick the SQLite or Postgres SQL for a query. When `pg` is `None` and the pool is
/// Postgres, the SQLite string is rebound (`?` → `$n`). Use `Some(pg)` only when the
/// two dialects differ beyond placeholders.
fn sql_for<'a>(pool: &DbPool, sqlite: &'a str, pg: Option<&'a str>) -> Cow<'a, str> {
    match pool {
        DbPool::Sqlite(_) => Cow::Borrowed(sqlite),
        DbPool::Postgres(_) => match pg {
            Some(pg) => Cow::Borrowed(pg),
            None => Cow::Owned(pg_rebind(sqlite)),
        },
    }
}

/// Run `$body` against whichever concrete pool the enum holds. The body is expanded
/// once per variant, so `p` is `&SqlitePool` in one arm and `&PgPool` in the other.
/// The body must return a database-independent type (decode rows / map results inside).
macro_rules! dispatch {
    ($pool:expr, |$p:ident| $body:expr) => {
        match $pool {
            DbPool::Sqlite($p) => $body,
            DbPool::Postgres($p) => $body,
        }
    };
}

// -- Monitors --

pub async fn list_monitors(pool: &DbPool) -> Result<Vec<Monitor>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT * FROM monitors ORDER BY created_at DESC",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Monitor>(&sql).fetch_all(p).await
    })
}

pub async fn get_monitor(pool: &DbPool, id: &str) -> Result<Option<Monitor>, sqlx::Error> {
    let sql = sql_for(pool, "SELECT * FROM monitors WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Monitor>(&sql)
            .bind(id)
            .fetch_optional(p)
            .await
    })
}

pub async fn create_monitor(pool: &DbPool, input: &CreateMonitor) -> Result<Monitor, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let headers = input.headers.as_deref().unwrap_or("{}");
    let sql = sql_for(
        pool,
        "INSERT INTO monitors (id, name, type, url, interval_secs, timeout_secs, method, headers, body, script)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
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
            .execute(p)
            .await
            .map(|_| ())
    })?;
    get_monitor(pool, &id).await.map(|m| m.unwrap())
}

pub async fn update_monitor(
    pool: &DbPool,
    id: &str,
    input: &UpdateMonitor,
) -> Result<Option<Monitor>, sqlx::Error> {
    let existing = get_monitor(pool, id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let e = existing.unwrap();

    let sql = sql_for(
        pool,
        "UPDATE monitors SET name=?, url=?, interval_secs=?, timeout_secs=?, method=?, headers=?, body=?, script=?, active=?, updated_at=datetime('now') WHERE id=?",
        Some(
            "UPDATE monitors SET name=$1, url=$2, interval_secs=$3, timeout_secs=$4, method=$5, headers=$6, body=$7, script=$8, active=$9, updated_at=to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') WHERE id=$10",
        ),
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(input.name.as_deref().unwrap_or(&e.name))
            .bind(input.url.as_deref().unwrap_or(&e.url))
            .bind(input.interval_secs.unwrap_or(e.interval_secs))
            .bind(input.timeout_secs.unwrap_or(e.timeout_secs))
            .bind(
                input
                    .method
                    .as_deref()
                    .unwrap_or(e.method.as_deref().unwrap_or("GET")),
            )
            .bind(
                input
                    .headers
                    .as_deref()
                    .unwrap_or(e.headers.as_deref().unwrap_or("{}")),
            )
            .bind(input.body.as_deref().or(e.body.as_deref()))
            .bind(input.script.as_deref().or(e.script.as_deref()))
            .bind(input.active.unwrap_or(e.active))
            .bind(id)
            .execute(p)
            .await
            .map(|_| ())
    })?;
    get_monitor(pool, id).await
}

pub async fn delete_monitor(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let sql = sql_for(pool, "DELETE FROM monitors WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(id)
            .execute(p)
            .await
            .map(|r| r.rows_affected() > 0)
    })
}

pub async fn update_monitor_status(
    pool: &DbPool,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    let sql = sql_for(
        pool,
        "UPDATE monitors SET current_status = ?, updated_at = datetime('now') WHERE id = ?",
        Some(
            "UPDATE monitors SET current_status = $1, updated_at = to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') WHERE id = $2",
        ),
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(status)
            .bind(id)
            .execute(p)
            .await
            .map(|_| ())
    })
}

// -- Checks --

pub async fn insert_check(
    pool: &DbPool,
    monitor_id: &str,
    status: &str,
    response_time_ms: Option<i64>,
    status_code: Option<i64>,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    let sql = sql_for(
        pool,
        "INSERT INTO checks (monitor_id, status, response_time_ms, status_code, error) VALUES (?, ?, ?, ?, ?)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(monitor_id)
            .bind(status)
            .bind(response_time_ms)
            .bind(status_code)
            .bind(error)
            .execute(p)
            .await
            .map(|_| ())
    })
}

pub async fn get_checks(
    pool: &DbPool,
    monitor_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Check>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT * FROM checks WHERE monitor_id = ? ORDER BY checked_at DESC LIMIT ? OFFSET ?",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Check>(&sql)
            .bind(monitor_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(p)
            .await
    })
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
    pool: &DbPool,
    monitor_id: &str,
    days: i64,
) -> Result<Vec<DailyUptime>, sqlx::Error> {
    // Compute the cutoff in Rust so the same SQL works on SQLite and Postgres.
    let cutoff = (Utc::now() - Duration::days(days))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let sql = sql_for(
        pool,
        "SELECT substr(checked_at, 1, 10) as day, COUNT(*) as total,
                SUM(CASE WHEN status = 'healthy' THEN 1 ELSE 0 END) as healthy,
                SUM(CASE WHEN status = 'sick' THEN 1 ELSE 0 END) as sick,
                SUM(CASE WHEN status = 'dead' THEN 1 ELSE 0 END) as dead
         FROM checks WHERE monitor_id = ? AND checked_at >= ?
         GROUP BY day ORDER BY day ASC",
        None,
    );
    dispatch!(pool, |p| {
        let rows = sqlx::query(&sql)
            .bind(monitor_id)
            .bind(&cutoff)
            .fetch_all(p)
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
    })
}

pub async fn get_uptime(
    pool: &DbPool,
    monitor_id: &str,
    period_hours: i64,
) -> Result<UptimeResponse, sqlx::Error> {
    // Compute the cutoff in Rust so the same SQL works on SQLite and Postgres.
    let cutoff = (Utc::now() - Duration::hours(period_hours))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let sql = sql_for(
        pool,
        "SELECT * FROM checks WHERE monitor_id = ? AND checked_at >= ? ORDER BY checked_at DESC",
        None,
    );
    let checks: Vec<Check> = dispatch!(pool, |p| {
        sqlx::query_as(&sql)
            .bind(monitor_id)
            .bind(&cutoff)
            .fetch_all(p)
            .await
    })?;

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

pub async fn list_notifications(pool: &DbPool) -> Result<Vec<Notification>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT * FROM notifications ORDER BY created_at DESC",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Notification>(&sql).fetch_all(p).await
    })
}

pub async fn create_notification(
    pool: &DbPool,
    input: &CreateNotification,
) -> Result<Notification, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let config = input.config.to_string();
    let sql = sql_for(
        pool,
        "INSERT INTO notifications (id, name, type, config, reminders_only) VALUES (?, ?, ?, ?, ?)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&id)
            .bind(&input.name)
            .bind(&input.type_)
            .bind(&config)
            .bind(input.reminders_only)
            .execute(p)
            .await
            .map(|_| ())
    })?;
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
    pool: &DbPool,
    id: &str,
    input: &UpdateNotification,
) -> Result<Option<Notification>, sqlx::Error> {
    let sel = sql_for(pool, "SELECT * FROM notifications WHERE id = ?", None);
    let existing = dispatch!(pool, |p| {
        sqlx::query_as::<_, Notification>(&sel)
            .bind(id)
            .fetch_optional(p)
            .await
    })?;
    if existing.is_none() {
        return Ok(None);
    }
    let e = existing.unwrap();

    let config = input
        .config
        .as_ref()
        .map(|c| c.to_string())
        .unwrap_or(e.config);
    let sql = sql_for(
        pool,
        "UPDATE notifications SET name=?, config=?, reminders_only=?, active=?, updated_at=datetime('now') WHERE id=?",
        Some(
            "UPDATE notifications SET name=$1, config=$2, reminders_only=$3, active=$4, updated_at=to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') WHERE id=$5",
        ),
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(input.name.as_deref().unwrap_or(&e.name))
            .bind(&config)
            .bind(input.reminders_only.unwrap_or(e.reminders_only))
            .bind(input.active.unwrap_or(e.active))
            .bind(id)
            .execute(p)
            .await
            .map(|_| ())
    })?;
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Notification>(&sel)
            .bind(id)
            .fetch_one(p)
            .await
    })
    .map(Some)
}

pub async fn delete_notification(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let sql = sql_for(pool, "DELETE FROM notifications WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(id)
            .execute(p)
            .await
            .map(|r| r.rows_affected() > 0)
    })
}

// -- Settings --

pub async fn get_setting(pool: &DbPool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let sql = sql_for(pool, "SELECT value FROM settings WHERE key = ?", None);
    dispatch!(pool, |p| {
        sqlx::query_scalar::<_, String>(&sql)
            .bind(key)
            .fetch_optional(p)
            .await
    })
}

pub async fn list_settings(pool: &DbPool) -> Result<Vec<Setting>, sqlx::Error> {
    let sql = sql_for(pool, "SELECT * FROM settings ORDER BY key", None);
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Setting>(&sql).fetch_all(p).await
    })
}

pub async fn upsert_setting(pool: &DbPool, input: &UpsertSetting) -> Result<(), sqlx::Error> {
    // `ON CONFLICT(key) DO UPDATE SET value = excluded.value` is valid in both dialects.
    let sql = sql_for(
        pool,
        "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&input.key)
            .bind(&input.value)
            .execute(p)
            .await
            .map(|_| ())
    })
}

// -- Users --

pub async fn get_user_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    let sql = sql_for(pool, "SELECT * FROM users WHERE username = ?", None);
    dispatch!(pool, |p| {
        sqlx::query_as::<_, User>(&sql)
            .bind(username)
            .fetch_optional(p)
            .await
    })
}

pub async fn list_users(pool: &DbPool) -> Result<Vec<UserInfo>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT id, username, must_change_password, created_at FROM users ORDER BY created_at ASC",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, UserInfo>(&sql).fetch_all(p).await
    })
}

pub async fn create_user(
    pool: &DbPool,
    username: &str,
    password_hash: &str,
) -> Result<UserInfo, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let sql = sql_for(
        pool,
        "INSERT INTO users (id, username, password_hash, must_change_password) VALUES (?, ?, ?, 1)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&id)
            .bind(username)
            .bind(password_hash)
            .execute(p)
            .await
            .map(|_| ())
    })?;
    Ok(UserInfo {
        id,
        username: username.to_string(),
        must_change_password: 1,
        created_at: String::new(),
    })
}

pub async fn update_user_password(
    pool: &DbPool,
    id: &str,
    new_hash: &str,
) -> Result<(), sqlx::Error> {
    let sql = sql_for(
        pool,
        "UPDATE users SET password_hash = ?, must_change_password = 0 WHERE id = ?",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(new_hash)
            .bind(id)
            .execute(p)
            .await
            .map(|_| ())
    })
}

pub async fn delete_user(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let sql = sql_for(pool, "DELETE FROM users WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(id)
            .execute(p)
            .await
            .map(|r| r.rows_affected() > 0)
    })
}

// -- Announcements --

pub async fn list_announcements(pool: &DbPool) -> Result<Vec<Announcement>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT * FROM announcements ORDER BY created_at DESC",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Announcement>(&sql).fetch_all(p).await
    })
}

pub async fn create_announcement(
    pool: &DbPool,
    input: &CreateAnnouncement,
) -> Result<Announcement, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let sql = sql_for(
        pool,
        "INSERT INTO announcements (id, title, text) VALUES (?, ?, ?)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&id)
            .bind(&input.title)
            .bind(&input.text)
            .execute(p)
            .await
            .map(|_| ())
    })?;
    Ok(Announcement {
        id,
        title: input.title.clone(),
        text: input.text.clone(),
        created_at: String::new(),
    })
}

// -- Incidents --

pub async fn list_incidents(pool: &DbPool, limit: i64) -> Result<Vec<Incident>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT * FROM incidents ORDER BY started_at DESC LIMIT ?",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Incident>(&sql)
            .bind(limit)
            .fetch_all(p)
            .await
    })
}

pub async fn create_incident(pool: &DbPool, monitor_id: &str) -> Result<Incident, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let sql = sql_for(
        pool,
        "INSERT INTO incidents (id, monitor_id) VALUES (?, ?)",
        None,
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&id)
            .bind(monitor_id)
            .execute(p)
            .await
            .map(|_| ())
    })?;
    let sel = sql_for(pool, "SELECT * FROM incidents WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query_as::<_, Incident>(&sel)
            .bind(&id)
            .fetch_one(p)
            .await
    })
}

pub async fn resolve_incident(pool: &DbPool, monitor_id: &str) -> Result<(), sqlx::Error> {
    let sql = sql_for(
        pool,
        "UPDATE incidents SET resolved_at = datetime('now'), status = 'resolved' WHERE monitor_id = ? AND resolved_at IS NULL",
        Some(
            "UPDATE incidents SET resolved_at = to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS'), status = 'resolved' WHERE monitor_id = $1 AND resolved_at IS NULL",
        ),
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(monitor_id)
            .execute(p)
            .await
            .map(|_| ())
    })
}

pub async fn delete_announcement(pool: &DbPool, id: &str) -> Result<bool, sqlx::Error> {
    let sql = sql_for(pool, "DELETE FROM announcements WHERE id = ?", None);
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(id)
            .execute(p)
            .await
            .map(|r| r.rows_affected() > 0)
    })
}

// -- Subscribers --

pub async fn add_subscriber(pool: &DbPool, email: &str) -> Result<bool, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let sql = sql_for(
        pool,
        "INSERT OR IGNORE INTO subscribers (id, email) VALUES (?, ?)",
        Some("INSERT INTO subscribers (id, email) VALUES ($1, $2) ON CONFLICT (email) DO NOTHING"),
    );
    dispatch!(pool, |p| {
        sqlx::query(&sql)
            .bind(&id)
            .bind(email)
            .execute(p)
            .await
            .map(|r| r.rows_affected() > 0)
    })
}

// -- Feed data --

pub async fn get_recent_incidents_with_monitors(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<(Incident, String)>, sqlx::Error> {
    let sql = sql_for(
        pool,
        "SELECT i.id, i.monitor_id, i.started_at, i.resolved_at, i.status, COALESCE(m.name, i.monitor_id) as monitor_name
         FROM incidents i
         LEFT JOIN monitors m ON m.id = i.monitor_id
         ORDER BY i.started_at DESC LIMIT ?",
        None,
    );
    dispatch!(pool, |p| {
        let rows = sqlx::query(&sql).bind(limit).fetch_all(p).await?;
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
    })
}
