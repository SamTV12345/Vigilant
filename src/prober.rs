// Probe engine — polls monitors from DB and writes results

use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use log::{debug, error, info};
use sqlx::SqlitePool;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::db::models::Monitor;
use crate::db::queries;
use crate::notifier::NotifierState;

pub async fn start(pool: SqlitePool, notifier: Arc<Mutex<NotifierState>>) {
    info!("probe engine started");
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let mut last_probe: HashMap<String, Instant> = HashMap::new();

    loop {
        interval.tick().await;

        let monitors = match queries::list_monitors(&pool).await {
            Ok(m) => m,
            Err(e) => {
                error!("failed to list monitors: {e}");
                continue;
            }
        };

        let now = Instant::now();
        let due: Vec<Monitor> = monitors
            .into_iter()
            .filter(|m| {
                if !m.active {
                    return false;
                }
                let interval = Duration::from_secs(m.interval_secs.max(1) as u64);
                last_probe
                    .get(&m.id)
                    .map_or(true, |last| now.duration_since(*last) >= interval)
            })
            .collect();

        for m in &due {
            last_probe.insert(m.id.clone(), now);
        }

        if due.is_empty() {
            continue;
        }

        debug!(
            "probing {} monitor(s): {:?}",
            due.len(),
            due.iter().map(|m| m.name.as_str()).collect::<Vec<_>>()
        );

        let pool = pool.clone();
        let notifier = notifier.clone();
        tokio::spawn(probe_all(pool, due, notifier));
    }
}

async fn probe_all(pool: SqlitePool, monitors: Vec<Monitor>, notifier: Arc<Mutex<NotifierState>>) {
    let mut set = JoinSet::new();

    for monitor in monitors {
        let pool = pool.clone();
        let notifier = notifier.clone();
        set.spawn(async move {
            let (status, response_time_ms, status_code, error) = probe_one(&monitor).await;

            if let Err(e) = queries::insert_check(
                &pool,
                &monitor.id,
                &status,
                response_time_ms,
                status_code,
                error.as_deref(),
            )
            .await
            {
                error!("failed to insert check for {}: {e}", monitor.id);
            }

            debug!(
                "{} → {} ({}ms)",
                monitor.name,
                status,
                response_time_ms.unwrap_or(0)
            );

            if status != monitor.current_status {
                if let Err(e) = queries::update_monitor_status(&pool, &monitor.id, &status).await {
                    error!("failed to update status for {}: {e}", monitor.id);
                }

                // Track incidents
                if monitor.current_status == "healthy" && (status == "sick" || status == "dead") {
                    if let Err(e) = queries::create_incident(&pool, &monitor.id).await {
                        error!("failed to create incident for {}: {e}", monitor.id);
                    }
                } else if (monitor.current_status == "sick" || monitor.current_status == "dead")
                    && status == "healthy"
                {
                    if let Err(e) = queries::resolve_incident(&pool, &monitor.id).await {
                        error!("failed to resolve incident for {}: {e}", monitor.id);
                    }
                }

                // Dispatch notifications on status transition
                notifier
                    .lock()
                    .await
                    .check_and_notify(&monitor.id, &monitor.name, &monitor.current_status, &status)
                    .await;
            }
        });
    }

    while set.join_next().await.is_some() {}
}

async fn probe_one(monitor: &Monitor) -> (String, Option<i64>, Option<i64>, Option<String>) {
    let start = SystemTime::now();

    let (is_healthy, is_sick, status_code, error) = match monitor.type_.as_str() {
        "http" | "https" => probe_http(monitor).await,
        "tcp" => probe_tcp(monitor).await,
        "icmp" => probe_icmp(monitor).await,
        "dns" => probe_dns(monitor).await,
        "script" => probe_script(monitor).await,
        _ => (false, false, None, Some("unknown probe type".into())),
    };

    let elapsed = start.elapsed().unwrap_or(Duration::ZERO);
    let rt_ms = Some(elapsed.as_millis() as i64);

    if is_healthy {
        ("healthy".into(), rt_ms, status_code, None)
    } else if is_sick {
        ("sick".into(), rt_ms, status_code, error)
    } else {
        ("dead".into(), rt_ms, status_code, error)
    }
}

/// Run a blocking closure on a dedicated OS thread to avoid
/// reqwest::blocking's internal tokio runtime from panicking on Drop
/// inside tokio::task::spawn_blocking.
async fn run_blocking<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.await.unwrap_or_else(|_| std::process::abort())
}

async fn probe_http(monitor: &Monitor) -> (bool, bool, Option<i64>, Option<String>) {
    let url = monitor.url.clone();
    let method = monitor.method.clone().unwrap_or_else(|| "GET".into());
    let timeout = monitor.timeout_secs.max(1) as u64;

    run_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .gzip(false)
            .redirect(reqwest::redirect::Policy::none())
            .build();

        let Ok(client) = client else {
            return (
                false,
                false,
                None,
                Some("failed to build HTTP client".into()),
            );
        };

        let result = match method.as_str() {
            "HEAD" => client.head(&url).send(),
            "POST" => client.post(&url).send(),
            "PUT" => client.put(&url).send(),
            "PATCH" => client.patch(&url).send(),
            _ => client.get(&url).send(),
        };

        match result {
            Ok(resp) => {
                let code = resp.status().as_u16() as i64;
                if (200..400).contains(&(code as u16)) {
                    (true, false, Some(code), None)
                } else if (400..500).contains(&(code as u16)) {
                    (false, true, Some(code), Some(format!("HTTP {code}")))
                } else {
                    (false, false, Some(code), Some(format!("HTTP {code}")))
                }
            }
            Err(e) => {
                let msg = if e.is_timeout() {
                    "timeout".into()
                } else if e.is_connect() {
                    format!(
                        "connection failed: {}",
                        e.status().map_or("unknown".into(), |s| s.to_string())
                    )
                } else {
                    format!("{e}")
                };
                (false, false, None, Some(msg))
            }
        }
    })
    .await
}

async fn probe_tcp(monitor: &Monitor) -> (bool, bool, Option<i64>, Option<String>) {
    let url = monitor.url.clone();
    let timeout = monitor.timeout_secs.max(1) as u64;

    tokio::task::spawn_blocking(move || {
        let Ok(addrs) = url.to_socket_addrs() else {
            return (false, false, None, Some("DNS resolution failed".into()));
        };

        for addr in addrs {
            if let Ok(stream) =
                std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(timeout))
            {
                drop(stream);
                return (true, false, None, None);
            }
        }
        (false, false, None, Some("connection refused".into()))
    })
    .await
    .unwrap_or((false, false, None, Some("task panicked".into())))
}

async fn probe_icmp(monitor: &Monitor) -> (bool, bool, Option<i64>, Option<String>) {
    let host = monitor.url.clone();
    let timeout = monitor.timeout_secs.max(1) as u64;

    tokio::task::spawn_blocking(move || {
        let Ok(addrs) = (host.as_str(), 0).to_socket_addrs() else {
            return (false, false, None, Some("DNS resolution failed".into()));
        };

        for addr in addrs {
            let ip = addr.ip();
            match ping::Ping::new(ip)
                .timeout(Duration::from_secs(timeout))
                .send()
            {
                Ok(_) => return (true, false, None, None),
                Err(_) => continue,
            }
        }
        (false, false, None, Some("no response".into()))
    })
    .await
    .unwrap_or((false, false, None, Some("task panicked".into())))
}

async fn probe_dns(monitor: &Monitor) -> (bool, bool, Option<i64>, Option<String>) {
    let host = monitor.url.clone();

    tokio::task::spawn_blocking(move || match (host.as_str(), 0).to_socket_addrs() {
        Ok(mut addrs) => {
            if addrs.next().is_some() {
                (true, false, None, None)
            } else {
                (false, false, None, Some("no addresses resolved".into()))
            }
        }
        Err(e) => (false, false, None, Some(format!("resolution failed: {e}"))),
    })
    .await
    .unwrap_or((false, false, None, Some("task panicked".into())))
}

async fn probe_script(monitor: &Monitor) -> (bool, bool, Option<i64>, Option<String>) {
    let script = monitor.script.clone().unwrap_or_default();

    tokio::task::spawn_blocking(move || {
        let opts = run_script::ScriptOptions::new();
        match run_script::run(&script, &Vec::<String>::new(), &opts) {
            Ok((0, _, _)) => (true, false, None, None),
            Ok((1, _, _)) => (false, true, None, Some("script exit code 1".into())),
            Ok((code, _, _)) => (false, false, None, Some(format!("script exit code {code}"))),
            Err(e) => (false, false, None, Some(format!("script error: {e}"))),
        }
    })
    .await
    .unwrap_or((false, false, None, Some("task panicked".into())))
}
