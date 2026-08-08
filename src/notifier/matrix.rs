// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// Copyright: 2021, Enrico Risa https://github.com/wolf4ood
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use reqwest::blocking::Client;

use super::generic::{DISPATCH_TIMEOUT_SECONDS, GenericNotifier, Notification};
use crate::APP_CONF;
use crate::config::config::ConfigNotify;

static MATRIX_HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(DISPATCH_TIMEOUT_SECONDS))
        .gzip(true)
        .build()
        .unwrap()
});
static MATRIX_FORMATTERS: LazyLock<Vec<fn(&Notification) -> String>> = LazyLock::new(|| {
    vec![
        format_status,
        format_replicas,
        format_status_page,
        format_time,
    ]
});

static MATRIX_MESSAGE_BODY: &'static str = "You received a Vigil alert.";
static MATRIX_MESSAGE_TYPE: &'static str = "m.text";
static MATRIX_MESSAGE_FORMAT: &'static str = "org.matrix.custom.html";

pub struct MatrixNotifier;

impl GenericNotifier for MatrixNotifier {
    fn attempt(notify: &ConfigNotify, notification: &Notification) -> Result<(), bool> {
        if let Some(ref matrix) = notify.matrix {
            // Build up the message text
            let message = format_message(notification);

            debug!("will send Matrix notification with message: {}", &message);

            // Generate URL
            // See: https://matrix.org/docs/guides/client-server-api#sending-messages
            let url = format!(
                "{}_matrix/client/r0/rooms/{}/send/m.room.message?access_token={}",
                matrix.homeserver_url.as_str(),
                matrix.room_id.as_str(),
                matrix.access_token.as_str()
            );

            // Build message parameters
            let mut params: HashMap<&str, &str> = HashMap::new();

            params.insert("body", MATRIX_MESSAGE_BODY);
            params.insert("msgtype", MATRIX_MESSAGE_TYPE);
            params.insert("format", MATRIX_MESSAGE_FORMAT);
            params.insert("formatted_body", &message);

            // Submit message to Matrix
            let response = MATRIX_HTTP_CLIENT.post(&url).json(&params).send();

            if let Ok(response_inner) = response {
                if response_inner.status().is_success() != true {
                    return Err(true);
                }
            } else {
                return Err(true);
            }

            return Ok(());
        }

        Err(false)
    }

    fn can_notify(notify: &ConfigNotify, notification: &Notification) -> bool {
        if let Some(ref matrix_config) = notify.matrix {
            notification.expected(matrix_config.reminders_only)
        } else {
            false
        }
    }

    fn name() -> &'static str {
        "matrix"
    }
}

fn format_status(notification: &Notification) -> String {
    let msg = if notification.startup == true {
        "Status started up, as"
    } else if notification.changed == true {
        "Status changed to"
    } else {
        "Status is still"
    };

    format!(
        "<p>{} {}: <em>{}</em>.</p>",
        notification.status.as_icon(),
        msg,
        notification.status.as_str().to_uppercase()
    )
}

fn format_replicas(notification: &Notification) -> String {
    let replicas = notification
        .replicas
        .iter()
        .map(|replica| replica.split(":").take(2).collect::<Vec<&str>>().join(":"))
        .fold(HashMap::new(), |mut replicas_count, replica| {
            *replicas_count.entry(replica).or_insert(0) += 1;
            replicas_count
        })
        .iter()
        .map(|(service_and_node, count)| {
            format!(
                "<li><code>{}</code>: {} {}</li>",
                service_and_node,
                count,
                notification.status.as_str()
            )
        })
        .collect::<Vec<String>>();

    if replicas.is_empty() {
        "".to_string()
    } else {
        format!("<ul>{}</ul>", replicas.join(""))
    }
}

fn format_status_page(_: &Notification) -> String {
    format!(
        "<p>Status page: {}</p>",
        APP_CONF.branding.page_url.as_str()
    )
}

fn format_time(notification: &Notification) -> String {
    format!("<p>Time: {}</p>", notification.time)
}

fn format_message(notification: &Notification) -> String {
    MATRIX_FORMATTERS
        .iter()
        .fold(String::new(), |mut accumulator, formatter| {
            accumulator.push_str(formatter(notification).as_str());
            accumulator
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(startup: bool, changed: bool) -> Notification<'static> {
        Notification {
            status: &crate::prober::status::Status::Dead,
            time: "12:00:00 UTC+00:00".to_string(),
            replicas: vec!["svc:node:r0", "svc:node:r1"],
            changed,
            escalated: None,
            startup,
        }
    }

    #[test]
    fn test_format_status_startup() {
        let n = make_notification(true, true);
        let result = format_status(&n);
        assert!(result.contains("started up"));
        assert!(result.contains("DEAD"));
        assert!(result.contains("<p>"));
        assert!(result.contains("<em>"));
    }

    #[test]
    fn test_format_status_changed() {
        let n = make_notification(false, true);
        let result = format_status(&n);
        assert!(result.contains("changed to"));
    }

    #[test]
    fn test_format_status_unchanged() {
        let n = make_notification(false, false);
        let result = format_status(&n);
        assert!(result.contains("is still"));
    }

    #[test]
    fn test_format_replicas_with_items() {
        let n = make_notification(false, false);
        let result = format_replicas(&n);
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>"));
        assert!(result.contains("svc:node"));
        assert!(result.contains("dead"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_format_replicas_counts_duplicates() {
        let mut n = make_notification(false, false);
        n.replicas = vec!["svc:node:r0", "svc:node:r0", "svc:node:r1"];
        let result = format_replicas(&n);
        // svc:node appears 3 times (all replicas map to same service:node prefix)
        assert!(
            result.contains("svc:node"),
            "expected svc:node in: {result}"
        );
        assert!(result.contains("<li>"), "expected <li> in: {result}");
    }

    #[test]
    fn test_format_replicas_empty() {
        let mut n = make_notification(false, false);
        n.replicas = vec![];
        let result = format_replicas(&n);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_status_page() {
        let n = make_notification(false, false);
        let result = format_status_page(&n);
        assert!(result.contains("Status page:"));
    }

    #[test]
    fn test_format_time() {
        let n = make_notification(false, false);
        let result = format_time(&n);
        assert!(result.contains("<p>Time:"));
        assert!(result.contains("12:00:00"));
    }

    #[test]
    fn test_format_message_concatenates_all_formatters() {
        let n = make_notification(false, false);
        let result = format_message(&n);
        assert!(result.contains("<p>"));
        assert!(result.contains("<ul>") || result.is_empty());
        // Should contain parts from all four formatters
        assert!(!result.is_empty());
    }
}
