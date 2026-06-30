use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClusterError {
    #[error("Node registration failed: {0}")]
    RegistrationFailed(String),
    #[error("Consensus split-brain or election failure: {0}")]
    ConsensusError(String),
    #[error("Scheduling allocation failed: {0}")]
    SchedulingFailed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Suspect,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RaftRole {
    Leader,
    Follower,
    Candidate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterNode {
    pub node_id: String,
    pub address: String,
    pub cpu_cores_total: u32,
    pub cpu_cores_used: u32,
    pub memory_bytes_total: u64,
    pub memory_bytes_used: u64,
    pub gpu_devices_total: u32,
    pub gpu_devices_used: u32,
    pub status: NodeStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchedulingRequest {
    pub workload_id: String,
    pub required_cpu_cores: u32,
    pub required_memory_bytes: u64,
    pub required_gpus: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub workload_id: String,
    pub assigned_node_id: String,
}

pub struct ClusterCoordinator {
    pub current_role: RaftRole,
    pub current_term: u64,
    pub nodes: HashMap<String, ClusterNode>,
}

impl ClusterCoordinator {
    pub fn new() -> Self {
        Self {
            current_role: RaftRole::Follower,
            current_term: 0,
            nodes: HashMap::new(),
        }
    }

    /// Registers a new host node discovered via gossip protocol.
    pub fn register_discovered_node(&mut self, node: ClusterNode) {
        println!("[Gossip Node Discovery] Registered node: {} at address: {}", node.node_id, node.address);
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// Implements Raft Leader Election State Transitions.
    pub fn trigger_election_timeout(&mut self, node_id: &str) -> Result<(), ClusterError> {
        self.current_role = RaftRole::Candidate;
        self.current_term += 1;
        println!(
            "[Raft Election] Election timeout reached. Node {} transitioning to Candidate. Starting Term: {}",
            node_id, self.current_term
        );
        
        // Simulating collection of votes (majority check)
        let total_votes = self.nodes.len() + 1; // including self
        let votes_received = (total_votes / 2) + 1; // Simple majority

        if votes_received > total_votes / 2 {
            self.current_role = RaftRole::Leader;
            println!("[Raft Election] Won majority vote. Transitioning to Leader for Term: {}", self.current_term);
        } else {
            self.current_role = RaftRole::Follower;
            return Err(ClusterError::ConsensusError("Split-brain or split vote encountered. Returning to Follower state.".to_string()));
        }

        Ok(())
    }

    /// Schedules workloads onto nodes using the First-Fit Decreasing (FFD) algorithm.
    pub fn schedule_workload(&mut self, request: SchedulingRequest) -> Result<ScheduleResult, ClusterError> {
        println!(
            "[Bin-Scheduler] Attempting placement for workload: {} (CPU: {}, Mem: {}MB, GPU: {})",
            request.workload_id,
            request.required_cpu_cores,
            request.required_memory_bytes / (1024 * 1024),
            request.required_gpus
        );

        // Sort nodes based on available capacity (Dynamic First-Fit Decreasing)
        let mut sorted_nodes: Vec<&mut ClusterNode> = self.nodes.values_mut().collect();
        sorted_nodes.sort_by(|a, b| {
            let available_mem_a = a.memory_bytes_total - a.memory_bytes_used;
            let available_mem_b = b.memory_bytes_total - b.memory_bytes_used;
            available_mem_b.cmp(&available_mem_a) // Sort descending
        });

        for node in sorted_nodes {
            let available_cpu = node.cpu_cores_total - node.cpu_cores_used;
            let available_mem = node.memory_bytes_total - node.memory_bytes_used;
            let available_gpu = node.gpu_devices_total - node.gpu_devices_used;

            if available_cpu >= request.required_cpu_cores
                && available_mem >= request.required_memory_bytes
                && available_gpu >= request.required_gpus
            {
                // Assign and update resources on the target node
                node.cpu_cores_used += request.required_cpu_cores;
                node.memory_bytes_used += request.required_memory_bytes;
                node.gpu_devices_used += request.required_gpus;

                println!(
                    "[Bin-Scheduler] Placed workload {} successfully on node: {}",
                    request.workload_id, node.node_id
                );

                return Ok(ScheduleResult {
                    workload_id: request.workload_id,
                    assigned_node_id: node.node_id.clone(),
                });
            }
        }

        Err(ClusterError::SchedulingFailed("No nodes found with sufficient CPU, Memory, or GPU capacity.".to_string()))
    }
}
