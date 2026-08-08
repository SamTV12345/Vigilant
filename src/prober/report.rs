// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::{Duration, SystemTime};

use super::states::{
    ServiceStatesProbeNodeRabbitMQ, ServiceStatesProbeNodeReplica,
    ServiceStatesProbeNodeReplicaLoad, ServiceStatesProbeNodeReplicaLoadQueue,
    ServiceStatesProbeNodeReplicaMetrics, ServiceStatesProbeNodeReplicaMetricsSystem,
    ServiceStatesProbeNodeReplicaReport,
};
use crate::prober::manager::STORE as PROBER_STORE;
use crate::prober::mode::Mode;
use crate::prober::status::Status;

pub enum HandleLoadError {
    InvalidLoad,
    WrongMode,
    NotFound,
}

pub enum HandleHealthError {
    WrongMode,
    NotFound,
}

pub enum HandleFlushError {
    WrongMode,
    NotFound,
}

pub fn validate_load_values(load_cpu: f32, load_ram: f32) -> bool {
    load_cpu >= 0.0 && load_ram >= 0.0
}

pub fn load_float_to_percent(value: f32) -> u16 {
    (value * 100.0).round() as u16
}

pub fn is_valid_mode_for_load(mode: &Mode) -> bool {
    *mode == Mode::Push
}

pub fn is_valid_mode_for_health(mode: &Mode) -> bool {
    *mode == Mode::Local
}

pub fn is_valid_mode_for_flush(mode: &Mode) -> bool {
    *mode == Mode::Push || *mode == Mode::Local
}

pub fn handle_load(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
    interval: u64,
    load_cpu: f32,
    load_ram: f32,
) -> Result<Option<ServiceStatesProbeNodeRabbitMQ>, HandleLoadError> {
    debug!(
        "load report handle: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    // Validate loads
    if !validate_load_values(load_cpu, load_ram) {
        return Err(HandleLoadError::InvalidLoad);
    }

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt push? Dont accept report
            if !is_valid_mode_for_load(&node.mode) {
                return Err(HandleLoadError::WrongMode);
            }

            // Acquire previous replica status + previous queue load status (follow-up values)
            let (status, mut metrics, mut load_queue);

            load_queue = ServiceStatesProbeNodeReplicaLoadQueue::default();

            if let Some(ref replica) = node.replicas.get(replica_id) {
                status = replica.status.to_owned();
                metrics = replica.metrics.to_owned();

                if let Some(ref replica_load) = replica.load {
                    load_queue = replica_load.queue.clone();
                }
            } else {
                status = Status::Healthy;
                metrics = ServiceStatesProbeNodeReplicaMetrics::default();
            }

            // Assign new system metrics
            metrics.system = Some(ServiceStatesProbeNodeReplicaMetricsSystem {
                cpu: load_float_to_percent(load_cpu),
                ram: load_float_to_percent(load_ram),
            });

            // Bump stored replica
            node.replicas.insert(
                replica_id.to_string(),
                ServiceStatesProbeNodeReplica {
                    status: status,
                    url: None,
                    script: None,
                    metrics: metrics,
                    load: Some(ServiceStatesProbeNodeReplicaLoad {
                        cpu: load_cpu,
                        ram: load_ram,
                        queue: load_queue,
                    }),
                    report: Some(ServiceStatesProbeNodeReplicaReport {
                        time: SystemTime::now(),
                        interval: Duration::from_secs(interval),
                    }),
                },
            );

            return Ok(node.rabbitmq.clone());
        }
    }

    warn!(
        "load report could not be stored: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleLoadError::NotFound)
}

pub fn handle_health(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
    interval: u64,
    health: &Status,
) -> Result<(), HandleHealthError> {
    debug!(
        "health report handle: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt local? Dont accept report
            if !is_valid_mode_for_health(&node.mode) {
                return Err(HandleHealthError::WrongMode);
            }

            // Bump stored replica
            node.replicas.insert(
                replica_id.to_string(),
                ServiceStatesProbeNodeReplica {
                    status: health.to_owned(),
                    url: None,
                    script: None,
                    metrics: ServiceStatesProbeNodeReplicaMetrics::default(),
                    load: None,
                    report: Some(ServiceStatesProbeNodeReplicaReport {
                        time: SystemTime::now(),
                        interval: Duration::from_secs(interval),
                    }),
                },
            );

            return Ok(());
        }
    }

    warn!(
        "health report could not be stored: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleHealthError::NotFound)
}

pub fn handle_flush(
    probe_id: &str,
    node_id: &str,
    replica_id: &str,
) -> Result<(), HandleFlushError> {
    debug!(
        "flush report handle: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    let mut store = PROBER_STORE.write().unwrap();

    if let Some(ref mut probe) = store.states.probes.get_mut(probe_id) {
        if let Some(ref mut node) = probe.nodes.get_mut(node_id) {
            // Mode isnt push or local? Dont accept report
            if !is_valid_mode_for_flush(&node.mode) {
                return Err(HandleFlushError::WrongMode);
            }

            return if node.replicas.shift_remove(replica_id).is_none() {
                Err(HandleFlushError::NotFound)
            } else {
                Ok(())
            };
        }
    }

    warn!(
        "load report could not be flushed: {}:{}:{}",
        probe_id, node_id, replica_id
    );

    Err(HandleFlushError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_load_valid() {
        assert!(validate_load_values(0.0, 0.0));
        assert!(validate_load_values(0.5, 0.8));
        assert!(validate_load_values(1.0, 1.0));
    }

    #[test]
    fn test_validate_load_negative_cpu() {
        assert!(!validate_load_values(-0.1, 0.5));
    }

    #[test]
    fn test_validate_load_negative_ram() {
        assert!(!validate_load_values(0.5, -0.1));
    }

    #[test]
    fn test_validate_load_both_negative() {
        assert!(!validate_load_values(-1.0, -2.0));
    }

    #[test]
    fn test_load_float_to_percent() {
        assert_eq!(load_float_to_percent(0.0), 0);
        assert_eq!(load_float_to_percent(0.5), 50);
        assert_eq!(load_float_to_percent(0.99), 99);
        assert_eq!(load_float_to_percent(1.0), 100);
    }

    #[test]
    fn test_load_float_to_percent_rounding() {
        assert_eq!(load_float_to_percent(0.666), 67);
        assert_eq!(load_float_to_percent(0.334), 33);
    }

    #[test]
    fn test_is_valid_mode_for_load() {
        assert!(is_valid_mode_for_load(&Mode::Push));
        assert!(!is_valid_mode_for_load(&Mode::Poll));
        assert!(!is_valid_mode_for_load(&Mode::Script));
        assert!(!is_valid_mode_for_load(&Mode::Local));
    }

    #[test]
    fn test_is_valid_mode_for_health() {
        assert!(is_valid_mode_for_health(&Mode::Local));
        assert!(!is_valid_mode_for_health(&Mode::Push));
        assert!(!is_valid_mode_for_health(&Mode::Poll));
        assert!(!is_valid_mode_for_health(&Mode::Script));
    }

    #[test]
    fn test_is_valid_mode_for_flush() {
        assert!(is_valid_mode_for_flush(&Mode::Push));
        assert!(is_valid_mode_for_flush(&Mode::Local));
        assert!(!is_valid_mode_for_flush(&Mode::Poll));
        assert!(!is_valid_mode_for_flush(&Mode::Script));
    }
}
