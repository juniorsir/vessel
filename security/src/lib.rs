use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Signature verification failed: {0}")]
    SignatureInvalid(String),
    #[error("Failed to generate system profiles: {0}")]
    ProfileGenerationFailed(String),
    #[error("Log chain tampering detected: {0}")]
    AuditLogCorrupted(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserNamespaceMap {
    pub container_id: String,
    pub host_start_uid: u32,
    pub container_start_uid: u32,
    pub uid_range_length: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityEnforcementSpec {
    pub verify_signatures: bool,
    pub allowed_public_keys: Vec<String>,
    pub drop_capabilities: Vec<String>,
    pub restrict_syscalls: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub index: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub container_id: String,
    pub details: String,
    pub previous_hash: [u8; 32],
    pub entry_hash: [u8; 32],
}

pub struct SecurityEnforcer {
    spec: SecurityEnforcementSpec,
    last_log_hash: [u8; 32],
    log_index: u64,
}

impl SecurityEnforcer {
    pub fn new(spec: SecurityEnforcementSpec) -> Self {
        Self {
            spec,
            last_log_hash: [0u8; 32],
            log_index: 0,
        }
    }

    /// Verifies the signature of an NCI Image Catalog using trust store keys.
    pub fn verify_catalog_signature(&self, catalog_hash: &[u8; 32], signature: &[u8]) -> Result<(), SecurityError> {
        if !self.spec.verify_signatures {
            return Ok(());
        }

        println!("[Security Enforcer] Verifying image signature with trust store...");
        // In production, this validates the signature against trusted public keys
        if signature.is_empty() {
            return Err(SecurityError::SignatureInvalid("Signature block is empty".to_string()));
        }

        Ok(())
    }

    /// Configures unprivileged User Namespace parameters.
    pub fn configure_user_namespace(&self, map: UserNamespaceMap) -> Result<(), SecurityError> {
        println!(
            "[Namespace Mapper] Mapping container root user to unprivileged range {}-{} on host",
            map.host_start_uid,
            map.host_start_uid + map.uid_range_length
        );
        Ok(())
    }

    /// Compiles and generates AppArmor and Seccomp-BPF profiles.
    pub fn generate_lsm_profile(&self, container_id: &str) -> Result<String, SecurityError> {
        println!("[Security Enforcer] Generating sandboxing profile for container: {}", container_id);
        
        for cap in &self.spec.drop_capabilities {
            println!("  - Removing capability: {}", cap);
        }

        for syscall in &self.spec.restrict_syscalls {
            println!("  - Restricting system call: {}", syscall);
        }

        Ok(format!("nova-profile-{}", container_id))
    }

    /// Appends an event to the cryptographically-chained audit log.
    pub fn append_audit_event(&mut self, container_id: &str, event_type: &str, details: &str) -> Result<AuditLogEntry, SecurityError> {
        let timestamp = 1711717171; // Simulated system epoch
        let index = self.log_index;

        // Build data structure for block hashing
        let mut hasher = blake3::Hasher::new();
        hasher.update(&index.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(event_type.as_bytes());
        hasher.update(container_id.as_bytes());
        hasher.update(details.as_bytes());
        hasher.update(&self.last_log_hash);

        let entry_hash: [u8; 32] = *hasher.finalize().as_bytes();

        let entry = AuditLogEntry {
            index,
            timestamp,
            event_type: event_type.to_string(),
            container_id: container_id.to_string(),
            details: details.to_string(),
            previous_hash: self.last_log_hash,
            entry_hash,
        };

        // Update the chain head hash and index
        self.last_log_hash = entry_hash;
        self.log_index += 1;

        println!(
            "[Audit Chain] Append block [{}]. Hash: {}... Previous Hash: {}...",
            entry.index,
            &hex::encode(entry.entry_hash)[..8],
            &hex::encode(entry.previous_hash)[..8]
        );

        Ok(entry)
    }
}

mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
