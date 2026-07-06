use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Patra {
    #[serde(rename = "Mool", default)]
    pub mool: String,
    
    #[serde(rename = "Karya", default)]
    pub karya: String,
    
    #[serde(rename = "Smriti", default = "default_memory")]
    pub smriti: String,
    
    #[serde(rename = "Shakti", default = "default_cpu")]
    pub shakti: f32,
    
    // Privacy & Networking Subsystems
    #[serde(rename = "Gupt")]
    pub gupt: Option<String>,
    
    #[serde(rename = "Chhadm")]
    pub chhadm: Option<String>,
    
    #[serde(rename = "Sangjna", default = "default_name")]
    pub sangjna: String,
}

impl Patra {
    /// Loads and deserializes a Patra manifest from disk (supports both YAML and JSON).
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest file at {:?}: {}", path, e))?;

        // serde_yaml parses both YAML and JSON syntax natively
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse Patra manifest in {:?}: {}", path, e))
    }
}

fn default_memory() -> String {
    "1GB".to_string()
}

fn default_cpu() -> f32 {
    1.0
}

fn default_name() -> String {
    "vessel-node".to_string()
}
