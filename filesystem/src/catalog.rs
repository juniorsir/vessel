use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NciCatalog {
    pub nci_version: String,
    pub image_metadata: ImageMetadata,
    pub encryption_envelope: Option<EncryptionEnvelope>,
    pub filesystem_root: FilesystemNode,
    pub ai_stream_profile: Option<AiStreamProfile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageMetadata {
    pub architecture: String,
    pub os: String,
    pub created: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptionEnvelope {
    pub encrypted_session_key: String,
    pub key_recipient_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FilesystemNode {
    Directory {
        name: String,
        permissions: u32,
        uid: u32,
        gid: u32,
        children: Vec<FilesystemNode>,
    },
    File {
        name: String,
        permissions: u32,
        uid: u32,
        gid: u32,
        size: u64,
        chunks: Vec<ChunkMapping>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkMapping {
    pub hash: String,
    pub offset: u64,
    pub length: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiStreamProfile {
    pub access_prediction_model: String,
    pub prefetch_chunks: Vec<String>,
}

impl NciCatalog {
    /// Deserializes a Catalog from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Serializes the Catalog into a formatted JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
