use std::{collections::HashMap, ops::{Deref, DerefMut}, path::PathBuf, sync::{Arc, RwLock}};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
struct Data(Arc<ron::Value>);
impl Data {
    fn new(value: ron::Value) -> Self {
        Self(Arc::new(value))
    }
}
impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = ron::Value::deserialize(deserializer)?;
        Ok(Data(Arc::new(value)))
    }
}
impl Deref for Data {
    type Target = ron::Value;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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
    pub prefix: char,
    pub permissions: u64,
    pub owners: Vec<String>,
    pub components: HashMap<String, Data>,
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
        match std::fs::write(self.filepath, str_config) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to write {}: {}", self.filepath.to_string_lossy(), e.to_string())),
        }
    }
    pub fn register<S: AsRef<str>>(&mut self, name: S) -> Data {
        let name = name.as_ref();
        if self.components.contains_key(name) {
            self.components.get(name).unwrap().clone()
        } else {
            let value = Data::new(ron::Value::Unit);
            self.components.insert(name.to_string(), value.clone());
            value
        }
    }
}