//! Configuration de l'application
//! 
//! Un fichier config.json est lu et utilisé pour configurer l'application.
//! Celui ci contient des informations pour le client.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration de l'application
/// 
/// Actuellement, contient l'intégralité des données de l'application.
/// 
/// Le format choisi pour le fichier de configuration est le [json] pour l'interopérabilité. 
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
        let mut config: Config = match serde_json::from_str(&str_config) {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to parse {}: {}", filepath.as_ref().to_string_lossy(), e.to_string())),
        };
        config.filepath = filepath.as_ref().to_path_buf();
        Ok(config)
    }
}