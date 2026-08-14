// Notifier engine — status transition detection and dispatch

use std::collections::HashMap;
use std::time::Duration;

use lettre::message::Mailbox;
use lettre::{Message, SmtpTransport, Transport};
use log::{debug, error, info, warn};
use serde::Serialize;
use serde_json::Value;
use crate::db::DbPool;
use crate::db::models::Notification;

pub struct NotifierState {
    pool: DbPool,
    last_known: HashMap<String, String>, // monitor_id -> last status
}

impl NotifierState {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            last_known: HashMap::new(),
        }
    }

    /// Called after each probe result when status transitioned.
    /// `db_current_status` is the status the monitor had in the DB before this probe,
    /// used as fallback when this is the first probe seen by this notifier instance.
    pub async fn check_and_notify(
        &mut self,
        monitor_id: &str,
        monitor_name: &str,
        db_current_status: &str,
        new_status: &str,
    ) {
        let old_status = self
            .last_known
            .get(monitor_id)
            .cloned()
            .unwrap_or_else(|| db_current_status.to_string());
        if old_status == new_status {
            return;
        }

        // Status transition detected
        info!(
            "status transition: {} {} -> {}",
            monitor_name, old_status, new_status
        );

        self.last_known
            .insert(monitor_id.to_string(), new_status.to_string());

        // Load active notifications
        let notifications = match crate::db::queries::list_notifications(&self.pool).await {
            Ok(n) => n,
            Err(e) => {
                error!("failed to load notifications: {e}");
                return;
            }
        };

        let active: Vec<&Notification> = notifications.iter().filter(|n| n.active).collect();
        if active.is_empty() {
            debug!("no active notification channels — skipping dispatch");
            return;
        }

        // Dispatch each notification in its own OS thread to avoid
        // reqwest::blocking's internal tokio runtime from conflicting
        // with our async runtime.
        for notify in &active {
            let monitor_name = monitor_name.to_string();
            let old_status = old_status.clone();
            let new_status = new_status.to_string();
            let config_str = notify.config.clone();
            let type_ = notify.type_.clone();
            let notify_id = notify.id.clone();
            std::thread::spawn(move || {
                let config: Value = match serde_json::from_str(&config_str) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("invalid config for notification {notify_id}: {e}");
                        return;
                    }
                };

                match type_.as_str() {
                    "webhook" => send_webhook(&config, &monitor_name, &old_status, &new_status),
                    "slack" => send_slack(&config, &monitor_name, &old_status, &new_status),
                    "email" => send_email(&config, &monitor_name, &old_status, &new_status),
                    "telegram" => send_telegram(&config, &monitor_name, &old_status, &new_status),
                    "twilio" => send_twilio(&config, &monitor_name, &old_status, &new_status),
                    "pushover" => send_pushover(&config, &monitor_name, &old_status, &new_status),
                    "gotify" => send_gotify(&config, &monitor_name, &old_status, &new_status),
                    "zulip" => send_zulip(&config, &monitor_name, &old_status, &new_status),
                    "matrix" => send_matrix(&config, &monitor_name, &old_status, &new_status),
                    "webex" => send_webex(&config, &monitor_name, &old_status, &new_status),
                    _ => warn!("unknown notifier type '{type_}'"),
                }
            });
        }
    }
}

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .gzip(true)
        .build()
        .expect("build http client")
}

fn slack_color(status: &str) -> &str {
    match status {
        "healthy" => "good",
        "sick" | "partial" => "warning",
        _ => "danger",
    }
}

fn status_emoji(status: &str) -> &str {
    match status {
        "healthy" => "✅",
        "sick" | "partial" => "⚠️",
        _ => "🔴",
    }
}

fn transition_msg(monitor_name: &str, old_status: &str, new_status: &str) -> String {
    format!(
        "{} {} → {}",
        monitor_name,
        status_emoji(old_status),
        status_emoji(new_status)
    )
}

// -- Webhook notifier --

#[derive(Serialize)]
struct WebhookPayload {
    #[serde(rename = "type")]
    event_type: String,
    monitor: String,
    old_status: String,
    new_status: String,
    time: String,
}

fn send_webhook(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let hook_url = match config.get("hook_url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            warn!("webhook notification missing hook_url");
            return;
        }
    };

    let payload = WebhookPayload {
        event_type: "changed".into(),
        monitor: monitor_name.into(),
        old_status: old_status.into(),
        new_status: new_status.into(),
        time: chrono::Utc::now().to_rfc3339(),
    };

    match http_client().post(hook_url).json(&payload).send() {
        Ok(resp) if resp.status().is_success() => {
            info!("webhook sent for {monitor_name}: {old_status} -> {new_status}");
        }
        Ok(resp) => {
            warn!("webhook failed for {monitor_name}: HTTP {}", resp.status());
        }
        Err(e) => {
            error!("webhook error for {monitor_name}: {e}");
        }
    }
}

// -- Slack notifier --

#[derive(Serialize)]
struct SlackPayload {
    text: String,
    attachments: Vec<SlackAttachment>,
}

#[derive(Serialize)]
struct SlackAttachment {
    fallback: String,
    color: String,
    fields: Vec<SlackField>,
}

#[derive(Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

fn send_slack(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(hook_url) = config.get("hook_url").and_then(|v| v.as_str()) else {
        warn!("slack notification missing hook_url");
        return;
    };
    let mention = config
        .get("mention_channel")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let color = slack_color(new_status);
    let message = format!("*{monitor_name}* status changed: `{old_status}` → `{new_status}`");
    let text = if mention {
        format!("<!channel> {message}")
    } else {
        message.clone()
    };

    let payload = SlackPayload {
        text,
        attachments: vec![SlackAttachment {
            fallback: message.clone(),
            color: color.into(),
            fields: vec![
                SlackField {
                    title: "Monitor".into(),
                    value: monitor_name.into(),
                    short: true,
                },
                SlackField {
                    title: "Status".into(),
                    value: format!("{old_status} → {new_status}"),
                    short: true,
                },
                SlackField {
                    title: "Time".into(),
                    value: chrono::Utc::now().to_rfc3339(),
                    short: false,
                },
            ],
        }],
    };

    match http_client().post(hook_url).json(&payload).send() {
        Ok(resp) if resp.status().is_success() => info!("slack sent for {monitor_name}"),
        Ok(resp) => warn!("slack failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("slack error for {monitor_name}: {e}"),
    }
}

// -- Email (SMTP) --

fn send_email(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(host) = config.get("smtp_host").and_then(|v| v.as_str()) else {
        warn!("email missing smtp_host");
        return;
    };
    let port = config
        .get("smtp_port")
        .and_then(|v| v.as_u64())
        .unwrap_or(587) as u16;
    let username = config
        .get("smtp_username")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let password = config
        .get("smtp_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let from = config
        .get("from_email")
        .and_then(|v| v.as_str())
        .unwrap_or("vigilant@localhost");
    let Some(to) = config.get("to_email").and_then(|v| v.as_str()) else {
        warn!("email missing to_email");
        return;
    };

    let subject = format!("[{new_status}] {monitor_name}");
    let body = format!(
        "{monitor_name} status changed: {old_status} → {new_status}\n\nTime: {}",
        chrono::Utc::now().to_rfc3339()
    );

    let Ok(from_mbox) = from.parse::<Mailbox>() else {
        warn!("invalid from_email");
        return;
    };
    let Ok(to_mbox) = to.parse::<Mailbox>() else {
        warn!("invalid to_email");
        return;
    };
    let Ok(msg) = Message::builder()
        .from(from_mbox)
        .to(to_mbox)
        .subject(&subject)
        .body(body)
    else {
        warn!("failed to build email");
        return;
    };

    let mailer = if username.is_empty() {
        SmtpTransport::builder_dangerous(host).port(port).build()
    } else {
        SmtpTransport::builder_dangerous(host)
            .port(port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                username.into(),
                password.into(),
            ))
            .build()
    };

    match mailer.send(&msg) {
        Ok(_) => info!("email sent for {monitor_name}"),
        Err(e) => warn!("email failed for {monitor_name}: {e}"),
    }
}

// -- Telegram --

fn send_telegram(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(token) = config.get("bot_token").and_then(|v| v.as_str()) else {
        warn!("telegram missing bot_token");
        return;
    };
    let Some(chat_id) = config.get("chat_id").and_then(|v| v.as_str()) else {
        warn!("telegram missing chat_id");
        return;
    };

    let text = format!(
        "{} {monitor_name}\nStatus: {old_status} → {new_status}",
        status_emoji(new_status)
    );
    let payload = serde_json::json!({"chat_id": chat_id, "text": text, "parse_mode": "HTML"});

    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    match http_client().post(&url).json(&payload).send() {
        Ok(resp) if resp.status().is_success() => info!("telegram sent for {monitor_name}"),
        Ok(resp) => warn!("telegram failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("telegram error for {monitor_name}: {e}"),
    }
}

// -- Twilio SMS --

fn send_twilio(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(sid) = config.get("account_sid").and_then(|v| v.as_str()) else {
        warn!("twilio missing account_sid");
        return;
    };
    let Some(token) = config.get("auth_token").and_then(|v| v.as_str()) else {
        warn!("twilio missing auth_token");
        return;
    };
    let Some(from) = config.get("from").and_then(|v| v.as_str()) else {
        warn!("twilio missing from");
        return;
    };
    let Some(to) = config.get("to").and_then(|v| v.as_str()) else {
        warn!("twilio missing to");
        return;
    };

    let body = format!("{monitor_name} {old_status} → {new_status}");
    let params = [("From", from), ("To", to), ("Body", &body)];

    let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json");
    match http_client()
        .post(&url)
        .basic_auth(sid, Some(token))
        .form(&params)
        .send()
    {
        Ok(resp) if resp.status().is_success() => info!("twilio sent for {monitor_name}"),
        Ok(resp) => warn!("twilio failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("twilio error for {monitor_name}: {e}"),
    }
}

// -- Pushover --

#[derive(Serialize)]
struct PushoverPayload {
    token: String,
    user: String,
    message: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<String>,
}

fn send_pushover(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(user) = config.get("user_key").and_then(|v| v.as_str()) else {
        warn!("pushover missing user_key");
        return;
    };
    let Some(token) = config.get("api_token").and_then(|v| v.as_str()) else {
        warn!("pushover missing api_token");
        return;
    };
    let device = config
        .get("device")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let payload = PushoverPayload {
        token: token.into(),
        user: user.into(),
        message: format!("{old_status} → {new_status}"),
        title: format!("{monitor_name}"),
        device,
    };

    match http_client()
        .post("https://api.pushover.net/1/messages.json")
        .json(&payload)
        .send()
    {
        Ok(resp) if resp.status().is_success() => info!("pushover sent for {monitor_name}"),
        Ok(resp) => warn!("pushover failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("pushover error for {monitor_name}: {e}"),
    }
}

// -- Gotify --

fn send_gotify(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(server) = config.get("server_url").and_then(|v| v.as_str()) else {
        warn!("gotify missing server_url");
        return;
    };
    let Some(token) = config.get("app_token").and_then(|v| v.as_str()) else {
        warn!("gotify missing app_token");
        return;
    };
    let priority = config.get("priority").and_then(|v| v.as_u64()).unwrap_or(5);

    let payload = serde_json::json!({
        "title": monitor_name,
        "message": format!("{old_status} → {new_status}"),
        "priority": priority,
    });

    let url = format!("{server}/message?token={token}");
    match http_client().post(&url).json(&payload).send() {
        Ok(resp) if resp.status().is_success() => info!("gotify sent for {monitor_name}"),
        Ok(resp) => warn!("gotify failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("gotify error for {monitor_name}: {e}"),
    }
}

// -- Zulip --

fn send_zulip(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(bot_email) = config.get("bot_email").and_then(|v| v.as_str()) else {
        warn!("zulip missing bot_email");
        return;
    };
    let Some(api_key) = config.get("api_key").and_then(|v| v.as_str()) else {
        warn!("zulip missing api_key");
        return;
    };
    let Some(site) = config.get("site_url").and_then(|v| v.as_str()) else {
        warn!("zulip missing site_url");
        return;
    };
    let msg_type = config
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("stream");
    let to = config
        .get("to")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    let topic = config
        .get("topic")
        .and_then(|v| v.as_str())
        .unwrap_or("Vigilant Alerts");

    let text = format!("**{monitor_name}** status changed: `{old_status}` → `{new_status}`");

    let url = format!("{site}/api/v1/messages");
    match http_client()
        .post(&url)
        .basic_auth(bot_email, Some(api_key))
        .form(&[
            ("type", msg_type),
            ("to", to),
            ("topic", topic),
            ("content", &text),
        ])
        .send()
    {
        Ok(resp) if resp.status().is_success() => info!("zulip sent for {monitor_name}"),
        Ok(resp) => warn!("zulip failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("zulip error for {monitor_name}: {e}"),
    }
}

// -- Matrix --

fn send_matrix(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(homeserver) = config.get("homeserver_url").and_then(|v| v.as_str()) else {
        warn!("matrix missing homeserver_url");
        return;
    };
    let Some(token) = config.get("access_token").and_then(|v| v.as_str()) else {
        warn!("matrix missing access_token");
        return;
    };
    let Some(room) = config.get("room_id").and_then(|v| v.as_str()) else {
        warn!("matrix missing room_id");
        return;
    };

    let body = transition_msg(monitor_name, old_status, new_status);
    let payload = serde_json::json!({
        "msgtype": "m.notice",
        "body": body,
    });

    let txn = uuid::Uuid::new_v4().to_string();
    let url = format!("{homeserver}/_matrix/client/v3/rooms/{room}/send/m.room.message/{txn}");
    match http_client()
        .put(&url)
        .bearer_auth(token)
        .json(&payload)
        .send()
    {
        Ok(resp) if resp.status().is_success() => info!("matrix sent for {monitor_name}"),
        Ok(resp) => warn!("matrix failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("matrix error for {monitor_name}: {e}"),
    }
}

// -- Cisco Webex --

fn send_webex(config: &Value, monitor_name: &str, old_status: &str, new_status: &str) {
    let Some(token) = config.get("bot_token").and_then(|v| v.as_str()) else {
        warn!("webex missing bot_token");
        return;
    };

    let room_id = config.get("room_id").and_then(|v| v.as_str());
    let to_email = config.get("to_person_email").and_then(|v| v.as_str());
    if room_id.is_none() && to_email.is_none() {
        warn!("webex missing room_id or to_person_email");
        return;
    }

    let markdown = format!(
        "**{monitor_name}** {emoji}  \n{old_status} → {new_status}",
        emoji = status_emoji(new_status)
    );

    let mut payload = serde_json::json!({ "markdown": markdown });
    if let Some(rid) = room_id {
        payload["roomId"] = serde_json::Value::String(rid.into());
    }
    if let Some(email) = to_email {
        payload["toPersonEmail"] = serde_json::Value::String(email.into());
    }

    match http_client()
        .post("https://webexapis.com/v1/messages")
        .bearer_auth(token)
        .json(&payload)
        .send()
    {
        Ok(resp) if resp.status().is_success() => info!("webex sent for {monitor_name}"),
        Ok(resp) => warn!("webex failed for {monitor_name}: HTTP {}", resp.status()),
        Err(e) => error!("webex error for {monitor_name}: {e}"),
    }
}
