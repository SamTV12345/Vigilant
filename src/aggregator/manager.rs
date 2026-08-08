// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::iter::FromIterator;
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, SystemTime};
use time;
use time::format_description::FormatItem;

use crate::APP_CONF;
use crate::config::config::ConfigNotifyReminderBackoffFunction;
use crate::notifier::generic::Notification;
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::prober::status::Status;

#[cfg(feature = "notifier-email")]
use crate::notifier::email::EmailNotifier;

#[cfg(feature = "notifier-twilio")]
use crate::notifier::twilio::TwilioNotifier;

#[cfg(feature = "notifier-slack")]
use crate::notifier::slack::SlackNotifier;

#[cfg(feature = "notifier-zulip")]
use crate::notifier::zulip::ZulipNotifier;

#[cfg(feature = "notifier-telegram")]
use crate::notifier::telegram::TelegramNotifier;

#[cfg(feature = "notifier-pushover")]
use crate::notifier::pushover::PushoverNotifier;

#[cfg(feature = "notifier-gotify")]
use crate::notifier::gotify::GotifyNotifier;

#[cfg(feature = "notifier-xmpp")]
use crate::notifier::xmpp::XMPPNotifier;

#[cfg(feature = "notifier-matrix")]
use crate::notifier::matrix::MatrixNotifier;

#[cfg(feature = "notifier-webex")]
use crate::notifier::webex::WebExNotifier;

#[cfg(feature = "notifier-webhook")]
use crate::notifier::webhook::WebHookNotifier;

#[allow(deprecated)]
static TIME_NOW_FORMATTER: LazyLock<Vec<FormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse(
        "[hour]:[minute]:[second] UTC[offset_hour sign:mandatory]:[offset_minute]",
    )
    .expect("invalid time format")
});

const AGGREGATE_INTERVAL_SECONDS: u64 = 10;

struct BumpedStates {
    status: Status,
    replicas: Vec<String>,
    changed: bool,
    escalated: Option<u16>,
    startup: bool,
}

fn should_notify_status_transition(old_status: &Status, new_status: &Status) -> bool {
    (old_status != &Status::Dead && new_status == &Status::Dead)
        || (old_status == &Status::Dead && new_status != &Status::Dead)
}

fn compute_node_status_min_replicas(
    node_status: Status,
    total_replicas: usize,
    dead_count: usize,
    min_available: Option<usize>,
) -> Status {
    if node_status == Status::Dead && dead_count > 0 && total_replicas > 0 {
        if let Some(minimum) = min_available {
            let available = total_replicas - dead_count;
            if available > 0 && available >= minimum {
                return Status::Partial;
            }
        }
    }
    node_status
}

fn compute_reminder_backoff_seconds(
    base_interval: u64,
    counter: u16,
    function: &ConfigNotifyReminderBackoffFunction,
) -> u64 {
    base_interval * (counter as u64).pow(*function as u32)
}

fn check_child_status(parent_status: &Status, child_status: &Status) -> Option<Status> {
    if child_status == &Status::Dead {
        Some(Status::Dead)
    } else if child_status == &Status::Sick && parent_status != &Status::Dead {
        Some(Status::Sick)
    } else if child_status == &Status::Partial && parent_status == &Status::Healthy {
        Some(Status::Partial)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- check_child_status tests ---

    #[test]
    fn test_child_dead_always_wins() {
        assert_eq!(
            check_child_status(&Status::Healthy, &Status::Dead),
            Some(Status::Dead)
        );
        assert_eq!(
            check_child_status(&Status::Sick, &Status::Dead),
            Some(Status::Dead)
        );
        assert_eq!(
            check_child_status(&Status::Dead, &Status::Dead),
            Some(Status::Dead)
        );
        assert_eq!(
            check_child_status(&Status::Partial, &Status::Dead),
            Some(Status::Dead)
        );
    }

    #[test]
    fn test_child_sick_promotes_when_parent_not_dead() {
        assert_eq!(
            check_child_status(&Status::Healthy, &Status::Sick),
            Some(Status::Sick)
        );
        assert_eq!(
            check_child_status(&Status::Partial, &Status::Sick),
            Some(Status::Sick)
        );
    }

    #[test]
    fn test_child_sick_does_not_promote_when_parent_dead() {
        assert_eq!(check_child_status(&Status::Dead, &Status::Sick), None);
    }

    #[test]
    fn test_child_partial_promotes_when_parent_healthy() {
        assert_eq!(
            check_child_status(&Status::Healthy, &Status::Partial),
            Some(Status::Partial)
        );
    }

    #[test]
    fn test_child_partial_does_not_promote_when_parent_not_healthy() {
        assert_eq!(check_child_status(&Status::Sick, &Status::Partial), None);
        assert_eq!(check_child_status(&Status::Dead, &Status::Partial), None);
        assert_eq!(check_child_status(&Status::Partial, &Status::Partial), None);
    }

    #[test]
    fn test_child_healthy_never_promotes() {
        assert_eq!(check_child_status(&Status::Healthy, &Status::Healthy), None);
        assert_eq!(check_child_status(&Status::Sick, &Status::Healthy), None);
        assert_eq!(check_child_status(&Status::Dead, &Status::Healthy), None);
        assert_eq!(check_child_status(&Status::Partial, &Status::Healthy), None);
    }

    // --- should_notify_status_transition tests ---

    #[test]
    fn test_should_notify_healthy_to_dead() {
        assert!(should_notify_status_transition(
            &Status::Healthy,
            &Status::Dead
        ));
    }

    #[test]
    fn test_should_notify_sick_to_dead() {
        assert!(should_notify_status_transition(
            &Status::Sick,
            &Status::Dead
        ));
    }

    #[test]
    fn test_should_notify_dead_to_sick() {
        assert!(should_notify_status_transition(
            &Status::Dead,
            &Status::Sick
        ));
    }

    #[test]
    fn test_should_notify_dead_to_healthy() {
        assert!(should_notify_status_transition(
            &Status::Dead,
            &Status::Healthy
        ));
    }

    #[test]
    fn test_should_notify_healthy_to_healthy() {
        assert!(!should_notify_status_transition(
            &Status::Healthy,
            &Status::Healthy
        ));
    }

    #[test]
    fn test_should_notify_sick_to_sick() {
        assert!(!should_notify_status_transition(
            &Status::Sick,
            &Status::Sick
        ));
    }

    #[test]
    fn test_should_notify_partial_to_dead() {
        assert!(should_notify_status_transition(
            &Status::Partial,
            &Status::Dead
        ));
    }

    #[test]
    fn test_should_notify_dead_to_partial() {
        assert!(should_notify_status_transition(
            &Status::Dead,
            &Status::Partial
        ));
    }

    // --- compute_node_status_min_replicas tests ---

    #[test]
    fn test_min_replicas_dead_without_min_available() {
        let result = compute_node_status_min_replicas(Status::Dead, 5, 2, None);
        assert_eq!(result, Status::Dead);
    }

    #[test]
    fn test_min_replicas_enough_available_becomes_partial() {
        let result = compute_node_status_min_replicas(Status::Dead, 5, 3, Some(2));
        assert_eq!(result, Status::Partial);
    }

    #[test]
    fn test_min_replicas_too_few_available_stays_dead() {
        let result = compute_node_status_min_replicas(Status::Dead, 5, 4, Some(2));
        assert_eq!(result, Status::Dead);
    }

    #[test]
    fn test_min_replicas_all_dead_stays_dead() {
        let result = compute_node_status_min_replicas(Status::Dead, 3, 3, Some(1));
        assert_eq!(result, Status::Dead);
    }

    #[test]
    fn test_min_replicas_not_dead_passes_through() {
        let result = compute_node_status_min_replicas(Status::Healthy, 5, 3, Some(1));
        assert_eq!(result, Status::Healthy);
        let result = compute_node_status_min_replicas(Status::Sick, 5, 3, Some(1));
        assert_eq!(result, Status::Sick);
        let result = compute_node_status_min_replicas(Status::Partial, 5, 3, Some(1));
        assert_eq!(result, Status::Partial);
    }

    #[test]
    fn test_min_replicas_no_dead_passes_through() {
        let result = compute_node_status_min_replicas(Status::Dead, 5, 0, Some(2));
        assert_eq!(result, Status::Dead);
    }

    #[test]
    fn test_min_replicas_zero_total() {
        let result = compute_node_status_min_replicas(Status::Dead, 0, 0, Some(1));
        assert_eq!(result, Status::Dead);
    }

    // --- compute_reminder_backoff_seconds tests ---

    #[test]
    fn test_backoff_none_always_one() {
        let f = ConfigNotifyReminderBackoffFunction::None;
        assert_eq!(compute_reminder_backoff_seconds(120, 1, &f), 120);
        assert_eq!(compute_reminder_backoff_seconds(120, 5, &f), 120);
        assert_eq!(compute_reminder_backoff_seconds(120, 10, &f), 120);
    }

    #[test]
    fn test_backoff_linear() {
        let f = ConfigNotifyReminderBackoffFunction::Linear;
        assert_eq!(compute_reminder_backoff_seconds(10, 1, &f), 10);
        assert_eq!(compute_reminder_backoff_seconds(10, 2, &f), 20);
        assert_eq!(compute_reminder_backoff_seconds(10, 3, &f), 30);
    }

    #[test]
    fn test_backoff_square() {
        let f = ConfigNotifyReminderBackoffFunction::Square;
        assert_eq!(compute_reminder_backoff_seconds(10, 1, &f), 10);
        assert_eq!(compute_reminder_backoff_seconds(10, 2, &f), 40);
        assert_eq!(compute_reminder_backoff_seconds(10, 3, &f), 90);
    }

    #[test]
    fn test_backoff_cubic() {
        let f = ConfigNotifyReminderBackoffFunction::Cubic;
        assert_eq!(compute_reminder_backoff_seconds(10, 1, &f), 10);
        assert_eq!(compute_reminder_backoff_seconds(10, 2, &f), 80);
    }
}

fn scan_and_bump_states() -> Option<BumpedStates> {
    let mut bumped_replicas = Vec::new();

    let mut store = PROBER_STORE.write().unwrap();

    let mut general_status = Status::Healthy;

    for (probe_id, probe) in store.states.probes.iter_mut() {
        debug!("aggregate probe: {}", probe_id);

        let mut probe_status = Status::Healthy;

        for (node_id, node) in probe.nodes.iter_mut() {
            debug!("aggregate node: {}:{}", probe_id, node_id);

            let mut node_status = Status::Healthy;

            let mut dead_replica_count = 0usize;

            for (replica_id, replica) in node.replicas.iter_mut() {
                let mut replica_status = Status::Healthy;

                // Process metrics
                match node.mode {
                    Mode::Push => {
                        // Compare delays and compute a new status?
                        if let Some(ref replica_report) = replica.report {
                            if let Ok(duration_since_report) =
                                SystemTime::now().duration_since(replica_report.time)
                            {
                                if duration_since_report
                                    >= (replica_report.interval
                                        + Duration::from_secs(APP_CONF.metrics.push_delay_dead))
                                {
                                    debug!(
                                        "replica: {}:{}:{} is dead because it didnt report in a while",
                                        probe_id, node_id, replica_id
                                    );

                                    replica_status = Status::Dead;
                                }
                            }
                        }

                        // Compare system load indices and compute a new status?
                        if replica_status == Status::Healthy {
                            if let Some(ref replica_load) = replica.load {
                                if (replica_load.cpu > APP_CONF.metrics.push_system_cpu_sick_above)
                                    || (replica_load.ram
                                        > APP_CONF.metrics.push_system_ram_sick_above)
                                {
                                    debug!(
                                        "replica: {}:{}:{} is sick because it is overloaded",
                                        probe_id, node_id, replica_id
                                    );

                                    replica_status = Status::Sick;
                                }
                            }
                        }

                        // Check RabbitMQ queue full marker?
                        if replica_status == Status::Healthy {
                            if let Some(ref replica_load) = replica.load {
                                if replica_load.queue.stalled == true {
                                    replica_status = Status::Dead;
                                } else if replica_load.queue.loaded == true {
                                    replica_status = Status::Sick;
                                }
                            }
                        }
                    }
                    Mode::Local => {
                        // Assign stored status by default ('local' nodes report their status \
                        //   themselves)
                        replica_status = replica.status.to_owned();

                        // Compare delays and compute a new status?
                        if let Some(ref replica_report) = replica.report {
                            if let Ok(duration_since_report) =
                                SystemTime::now().duration_since(replica_report.time)
                            {
                                if duration_since_report
                                    >= (replica_report.interval
                                        + Duration::from_secs(APP_CONF.metrics.local_delay_dead))
                                {
                                    debug!(
                                        "replica: {}:{}:{} is dead because it didnt report in a while",
                                        probe_id, node_id, replica_id
                                    );

                                    replica_status = Status::Dead;
                                }
                            }
                        }
                    }
                    _ => {
                        // Forward stored status (eg. 'poll' or 'script' nodes)
                        replica_status = replica.status.to_owned();
                    }
                }

                // Increment dead replica count?
                if replica_status == Status::Dead {
                    dead_replica_count += 1;
                }

                // Bump node status with worst replica status?
                if let Some(worst_status) = check_child_status(&node_status, &replica_status) {
                    node_status = worst_status;
                }

                debug!(
                    "aggregated status for replica: {}:{}:{} => {:?}",
                    probe_id, node_id, replica_id, replica_status
                );

                // Append bumped replica path?
                if replica_status == Status::Dead {
                    bumped_replicas.push(format!("{}:{}:{}", probe_id, node_id, replica_id));
                }

                replica.status = replica_status;
            }

            // Aggregate dead replicas into node status, respecting minimum available replicas?
            node_status = compute_node_status_min_replicas(
                node_status,
                node.replicas.len(),
                dead_replica_count,
                node.min_replicas_available,
            );

            // Bump probe status with worst node status?
            if let Some(worst_status) = check_child_status(&probe_status, &node_status) {
                probe_status = worst_status;
            }

            debug!(
                "aggregated status for node: {}:{} => {:?}",
                probe_id, node_id, node_status
            );

            node.status = node_status;
        }

        // Bump general status with worst node status?
        if let Some(worst_status) = check_child_status(&general_status, &probe_status) {
            general_status = worst_status;
        }

        debug!(
            "aggregated status for probe: {} => {:?}",
            probe_id, probe_status
        );

        probe.status = probe_status;
    }

    // Check if general status has changed
    let has_changed = store.states.status != general_status;

    // Check if should dispatch notification later (only if critical)
    // Allow for cases:
    //   - healthy >> dead
    //   - sick    >> dead
    //   - dead    >> sick
    //   - dead    >> healthy
    let mut should_notify = should_notify_status_transition(&store.states.status, &general_status);

    // Reset all counters whenever we are not dead (yet, stored status changed)
    if has_changed == true && general_status != Status::Dead {
        store.states.notifier.reminder_escalate_counter = 0;
        store.states.notifier.reminder_backoff_counter = 1;
    }

    // Check if should re-notify? (in case status did not change; only if dead)
    // Notice: this is used to send periodic reminders of downtime (ie. 'still down' messages)
    if has_changed == false && should_notify == false && general_status == Status::Dead {
        debug!("status unchanged, but may need to re-notify; checking");

        if let Some(ref notify) = APP_CONF.notify {
            match (store.notified, notify.reminder_interval) {
                (Some(last_notified), Some(reminder_interval)) => {
                    if let Ok(duration_since_notified) =
                        SystemTime::now().duration_since(last_notified)
                    {
                        // Notice: we use backoff counter all the time because if it is disabled, \
                        //   then the value is 1 at any time, thus not impacting the interval.
                        let reminder_escalate_counter =
                            store.states.notifier.reminder_escalate_counter;
                        let reminder_backoff_counter =
                            store.states.notifier.reminder_backoff_counter;
                        let reminder_ignore_until = store.states.notifier.reminder_ignore_until;
                        let reminder_interval_backoff =
                            Duration::from_secs(compute_reminder_backoff_seconds(
                                reminder_interval,
                                reminder_backoff_counter,
                                &notify.reminder_backoff_function,
                            ));

                        // Check if reminders should be ignored for now?
                        let should_ignore_reminders =
                            if let Some(reminder_ignore_until) = reminder_ignore_until {
                                SystemTime::now() < reminder_ignore_until
                            } else {
                                false
                            };

                        debug!(
                            "checking if should re-notify about unchanged status ({}s / {}x / {}↑ / {})",
                            reminder_interval_backoff.as_secs(),
                            reminder_escalate_counter,
                            reminder_backoff_counter,
                            if should_ignore_reminders == false {
                                "✓"
                            } else {
                                "✖"
                            }
                        );

                        // Duration since last notified exceeds reminder interval? Should re-notify
                        if duration_since_notified >= reminder_interval_backoff
                            && should_ignore_reminders == false
                        {
                            info!("should re-notify about unchanged status");

                            should_notify = true;

                            // Increment the escalate counter? (reminder escalation is enabled)
                            if notify.reminder_escalate == true
                                && reminder_escalate_counter < u16::MAX
                            {
                                store.states.notifier.reminder_escalate_counter += 1;

                                debug!(
                                    "incremented re-notify escalate counter to: {}",
                                    store.states.notifier.reminder_escalate_counter
                                );
                            }

                            // Increment the backoff counter? (a backoff function is set, \
                            //   therefore reminders backoff is enabled)
                            if notify.reminder_backoff_function
                                != ConfigNotifyReminderBackoffFunction::None
                                && store.states.notifier.reminder_backoff_counter
                                    < notify.reminder_backoff_limit
                            {
                                store.states.notifier.reminder_backoff_counter += 1;

                                debug!(
                                    "incremented re-notify backoff counter to: {} (limit: {})",
                                    store.states.notifier.reminder_backoff_counter,
                                    notify.reminder_backoff_limit
                                );
                            }
                        } else {
                            debug!(
                                "should not re-notify about unchanged status (interval: {})",
                                reminder_interval
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Bump stored values
    store.states.status = general_status.to_owned();
    store.states.date = Some(time_now_as_string());

    if should_notify == true {
        store.notified = Some(SystemTime::now());

        // Acquire escalated state (if non-zero)
        let escalated = if store.states.notifier.reminder_escalate_counter > 0 {
            Some(store.states.notifier.reminder_escalate_counter)
        } else {
            None
        };

        // Generate bumped states
        Some(BumpedStates {
            status: general_status,
            replicas: bumped_replicas,
            changed: has_changed,
            escalated: escalated,
            startup: false,
        })
    } else {
        None
    }
}

fn time_now_as_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&TIME_NOW_FORMATTER)
        .unwrap_or("?".to_string())
}

fn dispatch_startup_notification() {
    if let Some(ref conf_notify) = APP_CONF.notify {
        if conf_notify.startup_notification == true {
            debug!("sending aggregate startup notification...");

            notify(&BumpedStates {
                status: Status::Healthy,
                replicas: Vec::new(),
                changed: true,
                escalated: None,
                startup: true,
            });
        }
    }
}

fn notify(bumped_states: &BumpedStates) {
    let notification = Notification {
        status: &bumped_states.status,
        time: time_now_as_string(),
        replicas: Vec::from_iter(bumped_states.replicas.iter().map(String::as_str)),
        changed: bumped_states.changed,
        escalated: bumped_states.escalated,
        startup: bumped_states.startup,
    };

    if let Some(ref notify) = APP_CONF.notify {
        #[cfg(feature = "notifier-email")]
        Notification::dispatch::<EmailNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-twilio")]
        Notification::dispatch::<TwilioNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-slack")]
        Notification::dispatch::<SlackNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-zulip")]
        Notification::dispatch::<ZulipNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-telegram")]
        Notification::dispatch::<TelegramNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-pushover")]
        Notification::dispatch::<PushoverNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-gotify")]
        Notification::dispatch::<GotifyNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-xmpp")]
        Notification::dispatch::<XMPPNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-matrix")]
        Notification::dispatch::<MatrixNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-webex")]
        Notification::dispatch::<WebExNotifier>(notify, &notification).ok();

        #[cfg(feature = "notifier-webhook")]
        Notification::dispatch::<WebHookNotifier>(notify, &notification).ok();
    }
}

pub fn run() {
    // Notify that systems are healthy (when booting up aggregator)
    dispatch_startup_notification();

    // Start aggregate loop
    loop {
        debug!("running an aggregate operation...");

        // Should notify after bump?
        let bumped_states = scan_and_bump_states();

        if let Some(ref bumped_states_inner) = bumped_states {
            notify(bumped_states_inner);
        }

        info!(
            "ran aggregate operation (notified: {})",
            bumped_states.is_some()
        );

        // Hold for next aggregate run
        thread::sleep(Duration::from_secs(AGGREGATE_INTERVAL_SECONDS));
    }
}
