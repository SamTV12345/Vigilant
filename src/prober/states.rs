// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::{Duration, SystemTime};

use indexmap::IndexMap;

use super::mode::Mode;
use super::replica::ReplicaURL;
use super::status::Status;
use crate::config::{config::ConfigProbeServiceNodeHTTPMethod, regex::Regex};

#[derive(Serialize)]
pub struct ServiceStates {
    pub status: Status,
    pub date: Option<String>,
    pub probes: IndexMap<String, ServiceStatesProbe>,
    pub notifier: ServiceStatesNotifier,
}

#[derive(Serialize)]
pub struct ServiceStatesProbe {
    pub id: String,
    pub label: String,
    pub status: Status,
    pub nodes: IndexMap<String, ServiceStatesProbeNode>,
}

#[derive(Serialize)]
pub struct ServiceStatesProbeNode {
    pub status: Status,
    pub label: String,
    pub mode: Mode,
    pub replicas: IndexMap<String, ServiceStatesProbeNodeReplica>,
    #[serde(default)]
    #[serde(with = "http_serde::header_map")]
    pub http_headers: http::HeaderMap,
    pub http_method: Option<ConfigProbeServiceNodeHTTPMethod>,
    pub http_body: Option<String>,
    pub http_body_healthy_match: Option<Regex>,
    pub reveal_replica_name: bool,
    pub min_replicas_available: Option<usize>,
    pub link_url: Option<String>,
    pub link_label: Option<String>,
    pub rabbitmq: Option<ServiceStatesProbeNodeRabbitMQ>,
}

#[derive(Serialize)]
pub struct ServiceStatesProbeNodeReplica {
    pub status: Status,
    pub url: Option<ReplicaURL>,
    pub script: Option<String>,
    pub metrics: ServiceStatesProbeNodeReplicaMetrics,
    pub load: Option<ServiceStatesProbeNodeReplicaLoad>,
    pub report: Option<ServiceStatesProbeNodeReplicaReport>,
}

#[derive(Serialize, Clone)]
pub struct ServiceStatesProbeNodeRabbitMQ {
    pub queue: String,
    pub queue_nack_healthy_below: Option<u32>,
    pub queue_nack_dead_above: Option<u32>,
}

#[derive(Serialize, Clone, Default)]
pub struct ServiceStatesProbeNodeReplicaMetrics {
    pub latency: Option<u64>,
    pub system: Option<ServiceStatesProbeNodeReplicaMetricsSystem>,
    pub rabbitmq: Option<ServiceStatesProbeNodeReplicaMetricsRabbitMQ>,
}

#[derive(Serialize, Clone)]
pub struct ServiceStatesProbeNodeReplicaMetricsSystem {
    pub cpu: u16,
    pub ram: u16,
}

#[derive(Serialize, Clone, Default)]
pub struct ServiceStatesProbeNodeReplicaMetricsRabbitMQ {
    pub queue_ready: u32,
    pub queue_nack: u32,
}

#[derive(Serialize)]
pub struct ServiceStatesProbeNodeReplicaLoad {
    pub cpu: f32,
    pub ram: f32,
    pub queue: ServiceStatesProbeNodeReplicaLoadQueue,
}

#[derive(Serialize, Clone, Default)]
pub struct ServiceStatesProbeNodeReplicaLoadQueue {
    pub loaded: bool,
    pub stalled: bool,
}

#[derive(Serialize)]
pub struct ServiceStatesProbeNodeReplicaReport {
    pub time: SystemTime,
    pub interval: Duration,
}

#[derive(Serialize)]
pub struct ServiceStatesNotifier {
    pub reminder_escalate_counter: u16,
    pub reminder_backoff_counter: u16,
    pub reminder_ignore_until: Option<SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replica_metrics_default() {
        let metrics = ServiceStatesProbeNodeReplicaMetrics::default();
        assert!(metrics.latency.is_none());
        assert!(metrics.system.is_none());
        assert!(metrics.rabbitmq.is_none());
    }

    #[test]
    fn test_load_queue_default() {
        let queue = ServiceStatesProbeNodeReplicaLoadQueue::default();
        assert!(!queue.loaded);
        assert!(!queue.stalled);
    }

    #[test]
    fn test_rabbitmq_metrics_default() {
        let rmq = ServiceStatesProbeNodeReplicaMetricsRabbitMQ::default();
        assert_eq!(rmq.queue_ready, 0);
        assert_eq!(rmq.queue_nack, 0);
    }

    #[test]
    fn test_serialize_replica_metrics_default() {
        let metrics = ServiceStatesProbeNodeReplicaMetrics::default();
        let json = serde_json::to_string(&metrics).unwrap();
        // All optional fields should serialize to null
        assert!(json.contains("\"latency\":null"));
        assert!(json.contains("\"system\":null"));
        assert!(json.contains("\"rabbitmq\":null"));
    }

    #[test]
    fn test_serialize_load_queue() {
        let queue = ServiceStatesProbeNodeReplicaLoadQueue {
            loaded: true,
            stalled: false,
        };
        let json = serde_json::to_string(&queue).unwrap();
        assert!(json.contains("\"loaded\":true"));
        assert!(json.contains("\"stalled\":false"));
    }
}
