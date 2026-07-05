use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Patra {
    #[serde(rename = "Mool")]
    pub mool: String,
    #[serde(rename = "Karya")]
    pub karya: String,

    #[serde(rename = "Smriti", default = "default_smriti")]
    pub smriti: String,
    #[serde(rename = "Shakti", default = "default_shakti")]
    pub shakti: f32,
    #[serde(rename = "Kendra")]
    pub kendra: Option<String>,
    #[serde(rename = "Tejas")]
    pub tejas: Option<String>,
    #[serde(rename = "Vayu")]
    pub vayu: Option<String>,
    #[serde(rename = "Gati")]
    pub gati: Option<String>,
    #[serde(rename = "Dwar")]
    pub dwar: Option<String>,
    #[serde(rename = "Sanket")]
    pub sanket: Option<String>,

    #[serde(rename = "Sadasya")]
    pub sadasya: Option<String>,
    #[serde(rename = "Sangjna", default = "default_sangjna")]
    pub sangjna: String,
    #[serde(rename = "Suraksha")]
    pub suraksha: Option<String>,
    #[serde(rename = "Kavach")]
    pub kavach: Option<String>,
    #[serde(rename = "Adhikar")]
    pub adhikar: Option<String>,

    // 🔐 Privacy & Anonymity Subsystems
    #[serde(rename = "Gupt")]
    pub gupt: Option<String>,
    #[serde(rename = "Chhadm")]
    pub chhadm: Option<String>,
}

fn default_smriti() -> String { "1GB".to_string() }
fn default_shakti() -> f32 { 1.0 }
fn default_sangjna() -> String { "vessel-node".to_string() }

impl Patra {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read Patra manifest: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse Patra manifest YAML: {}", e))
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize Patra manifest: {}", e))?;
        fs::write(path, content)
            .map_err(|e| format!("Failed to write Patra manifest: {}", e))
    }
}
