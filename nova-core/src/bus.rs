use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum SystemEvent {
    ContainerCreated { id: String, runtime: String },
    ContainerStarted { id: String },
    SecurityViolation { id: String, alert_level: u8, rule: String },
    StorageMounted { mount_point: String },
    ClusterStateChanged { term: u64, leader: String },
}

/// A thread-safe message broker implementing the publish-subscribe pattern
/// across all active Nova subsystems.
#[derive(Clone)]
pub struct SystemBus {
    sender: broadcast::Sender<SystemEvent>,
}

impl SystemBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: SystemEvent) -> Result<usize, String> {
        self.sender.send(event).map_err(|e| e.to_string())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
}
