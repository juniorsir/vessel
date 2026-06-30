pub mod catalog;

// Re-export core structs for easier workspace import paths
pub use catalog::{NciCatalog, FilesystemNode, ChunkMapping, AiStreamProfile};
