// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::net::SocketAddr;
use std::path::PathBuf;

use super::config::ConfigNotifyReminderBackoffFunction;

pub fn server_log_level() -> String {
    "error".to_string()
}

pub fn server_inet() -> SocketAddr {
    "[::1]:8080".parse().unwrap()
}

pub fn server_workers() -> usize {
    4
}

pub fn server_mcp_server() -> bool {
    false
}

pub fn assets_path() -> PathBuf {
    PathBuf::from("./res/assets/")
}

pub fn branding_page_title() -> String {
    "Status Page".to_string()
}

pub fn metrics_poll_interval() -> u64 {
    120
}

pub fn metrics_poll_retry() -> u64 {
    2
}

pub fn metrics_poll_retry_wait() -> u64 {
    500
}

pub fn metrics_poll_http_status_healthy_above() -> u16 {
    200
}

pub fn metrics_poll_http_status_healthy_below() -> u16 {
    400
}

pub fn metrics_poll_delay_dead() -> u64 {
    10
}

pub fn metrics_poll_delay_sick() -> u64 {
    5
}

pub fn metrics_poll_parallelism() -> u16 {
    4
}

pub fn metrics_push_delay_dead() -> u64 {
    20
}

pub fn metrics_push_system_cpu_sick_above() -> f32 {
    0.99
}

pub fn metrics_push_system_ram_sick_above() -> f32 {
    0.99
}

pub fn metrics_script_interval() -> u64 {
    300
}

pub fn script_parallelism() -> u16 {
    2
}

pub fn metrics_local_delay_dead() -> u64 {
    40
}

pub fn notify_startup_notification() -> bool {
    true
}

pub fn notify_reminder_backoff_function() -> ConfigNotifyReminderBackoffFunction {
    ConfigNotifyReminderBackoffFunction::None
}

pub fn notify_reminder_backoff_limit() -> u16 {
    3
}

pub fn notify_reminder_escalate() -> bool {
    false
}

#[cfg(feature = "notifier-email")]
pub fn notify_email_smtp_host() -> String {
    "localhost".to_string()
}

#[cfg(feature = "notifier-email")]
pub fn notify_email_smtp_port() -> u16 {
    587
}

#[cfg(feature = "notifier-email")]
pub fn notify_email_smtp_encrypt() -> bool {
    true
}

#[cfg(feature = "notifier-slack")]
pub fn notify_slack_mention_channel() -> bool {
    false
}

pub fn notify_generic_reminders_only() -> bool {
    false
}

pub fn probe_service_node_reveal_replica_name() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_log_level() {
        assert_eq!(server_log_level(), "error");
    }

    #[test]
    fn test_server_inet() {
        let addr = server_inet();
        assert_eq!(addr.to_string(), "[::1]:8080");
    }

    #[test]
    fn test_server_workers() {
        assert_eq!(server_workers(), 4);
    }

    #[test]
    fn test_assets_path() {
        assert_eq!(assets_path(), PathBuf::from("./res/assets/"));
    }

    #[test]
    fn test_branding_page_title() {
        assert_eq!(branding_page_title(), "Status Page");
    }

    #[test]
    fn test_metrics_poll_interval() {
        assert_eq!(metrics_poll_interval(), 120);
    }

    #[test]
    fn test_metrics_poll_retry() {
        assert_eq!(metrics_poll_retry(), 2);
    }

    #[test]
    fn test_metrics_poll_retry_wait() {
        assert_eq!(metrics_poll_retry_wait(), 500);
    }

    #[test]
    fn test_metrics_poll_http_status_range() {
        assert_eq!(metrics_poll_http_status_healthy_above(), 200);
        assert_eq!(metrics_poll_http_status_healthy_below(), 400);
    }

    #[test]
    fn test_metrics_poll_delays() {
        assert_eq!(metrics_poll_delay_dead(), 10);
        assert_eq!(metrics_poll_delay_sick(), 5);
    }

    #[test]
    fn test_metrics_parallelism() {
        assert_eq!(metrics_poll_parallelism(), 4);
        assert_eq!(script_parallelism(), 2);
    }

    #[test]
    fn test_metrics_push() {
        assert_eq!(metrics_push_delay_dead(), 20);
        assert!((metrics_push_system_cpu_sick_above() - 0.99).abs() < 0.001);
        assert!((metrics_push_system_ram_sick_above() - 0.99).abs() < 0.001);
    }

    #[test]
    fn test_metrics_script_interval() {
        assert_eq!(metrics_script_interval(), 300);
    }

    #[test]
    fn test_metrics_local_delay_dead() {
        assert_eq!(metrics_local_delay_dead(), 40);
    }

    #[test]
    fn test_notify_defaults() {
        assert!(notify_startup_notification());
        assert!(!notify_reminder_escalate());
        assert_eq!(notify_reminder_backoff_limit(), 3);
        assert!(!notify_generic_reminders_only());
    }

    #[test]
    fn test_probe_service_node_reveal_replica_name() {
        assert!(!probe_service_node_reveal_replica_name());
    }
}
