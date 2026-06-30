use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait FilesystemLayer: Send + Sync {
    /// Mounts a content-addressable layer to a specific path using overlay structures.
    async fn mount_layer(&self, layer_hashes: &[String], target: &PathBuf) -> Result<(), String>;

    /// Unmounts and detaches active virtual filesystems cleanly.
    async fn unmount_layer(&self, target: &PathBuf) -> Result<(), String>;
}
