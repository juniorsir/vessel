use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to construct sandbox partition: {0}")]
    ConstructionError(String),
    #[error("Storage assembly failed: {0}")]
    StorageError(String),
    #[error("Internal runtime execution error: {0}")]
    ExecutionError(String),
    #[error("Plugin hook intercept failure: {0}")]
    PluginFailure(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SandboxType {
    OsNative,
    GVisor,
    MicroVM,
    Wasm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkloadSpec {
    pub id: String,
    pub sandbox_type: SandboxType,
    pub rootfs_path: PathBuf,
    pub exec_args: Vec<String>,
    pub limits: ResourceLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_bytes: u64,
    pub cpu_cores: f32,
}

/// Dynamic hook types that plugins can implement.
#[derive(Clone, Debug)]
pub enum HookType {
    PreCreate,
    PostStart,
    PreDestroy,
}

/// Hook interface for writing extensions to the runtime.
#[async_trait]
pub trait RuntimePlugin: Send + Sync {
    fn name(&self) -> &str;
    async fn call_hook(&self, hook: HookType, spec: &WorkloadSpec) -> Result<(), SandboxError>;
}

/// Abstract representation of an active, isolated sandbox.
#[async_trait]
pub trait Sandbox: Send + Sync {
    async fn construct(&mut self) -> Result<(), SandboxError>;
    async fn start(&mut self) -> Result<(), SandboxError>;
    async fn pause(&mut self) -> Result<(), SandboxError>;
    async fn resume(&mut self) -> Result<(), SandboxError>;
    async fn destroy(&mut self) -> Result<(), SandboxError>;
    async fn get_pid(&self) -> Option<u32>;
}
