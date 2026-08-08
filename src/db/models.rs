// Vigilant
// Database models
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_: String, // http, tcp, icmp, dns, script
    pub url: String,
    pub interval_secs: i64,
    pub timeout_secs: i64,
    pub method: Option<String>,
    pub headers: Option<String>, // JSON
    pub body: Option<String>,
    pub script: Option<String>,
    pub active: bool,
    pub current_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Check {
    pub id: i64,
    pub monitor_id: String,
    pub status: String,
    pub response_time_ms: Option<i64>,
    pub status_code: Option<i64>,
    pub error: Option<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub type_: String,
    pub config: String, // JSON
    pub reminders_only: bool,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Incident {
    pub id: String,
    pub monitor_id: String,
    pub started_at: String,
    pub resolved_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub must_change_password: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub must_change_password: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub text: String,
    pub created_at: String,
}

// -- API payloads --

#[derive(Debug, Deserialize)]
pub struct CreateMonitor {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
    #[serde(default = "default_interval")]
    pub interval_secs: i64,
    #[serde(default = "default_timeout")]
    pub timeout_secs: i64,
    pub method: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub script: Option<String>,
}

fn default_interval() -> i64 {
    60
}
fn default_timeout() -> i64 {
    10
}

#[derive(Debug, Deserialize)]
pub struct UpdateMonitor {
    pub name: Option<String>,
    pub url: Option<String>,
    pub interval_secs: Option<i64>,
    pub timeout_secs: Option<i64>,
    pub method: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub script: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotification {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub config: serde_json::Value,
    #[serde(default)]
    pub reminders_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotification {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub reminders_only: Option<bool>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub must_change_password: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub username: String,
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnnouncement {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct UptimeResponse {
    pub monitor_id: String,
    pub period_hours: i64,
    pub uptime_percent: f64,
    pub total_checks: i64,
    pub healthy_checks: i64,
    pub sick_checks: i64,
    pub dead_checks: i64,
}
