use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct SecurityProfile {
    pub seccomp_allow: Vec<String>,
    pub landlock_paths: Vec<String>,
    pub enforce_read_only_root: bool,
}

#[async_trait]
pub trait SecurityEngine: Send + Sync {
    /// Compiles high-level policy definitions to binary syscall filters.
    async fn generate_profile(&self, profile: &SecurityProfile) -> Result<Vec<u8>, String>;
    
    /// Attaches an eBPF audit probe to a target namespace or PID.
    async fn attach_audit_probe(&self, pid: u32) -> Result<(), String>;
}
