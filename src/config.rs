use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration de l'application
/// 
/// Actuellement, contient l'intégralité des données de l'application.
/// 
/// Le format choisi pour le fichier de configuration est le [ron]. 
/// Le format RON (Rusty Object Notation) est un format de données adapté pour le Rust 
/// et est pratique et lisible.
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub app_id: u64,
    pub prefix: char,
    pub permissions: u64,
    pub owners: Vec<String>,
    #[serde(skip)]
    filepath: PathBuf,
}

impl Config {
    pub fn load<P: AsRef<std::path::Path>>(filepath: P) -> Result<Self, String> {
        let str_config = match std::fs::read_to_string(filepath.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to read file {}: {}", filepath.as_ref().to_string_lossy(), e.to_string())),
        };
        let mut config: Config = match ron::from_str(&str_config) {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to parse {}: {}", filepath.as_ref().to_string_lossy(), e.to_string())),
        };
        config.filepath = filepath.as_ref().to_path_buf();
        Ok(config)
    }
    pub fn save(&self) -> Result<(), String> {
        let str_config = match ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default()) {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to serialize {}: {}", self.filepath.to_string_lossy(), e.to_string())),
        };
        match std::fs::write(&self.filepath, str_config) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to write {}: {}", self.filepath.to_string_lossy(), e.to_string())),
        }
    }
}