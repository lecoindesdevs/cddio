use std::{collections::HashMap, ops::{Deref, DerefMut}, path::PathBuf, sync::{Arc, RwLock, Weak, mpsc}};
use serde::{Deserialize, Serialize, Serializer};

pub struct DataGuard<'a> {
    data: &'a mut ron::Value,
    config_notify: mpsc::Sender<()>
}
impl<'a> Deref for DataGuard<'a> {
    type Target = ron::Value;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<'a> DerefMut for DataGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl<'a> Drop for DataGuard<'a> {
    fn drop(&mut self) {
        match self.config_notify.send(()) {
            Ok(_) => (),
            Err(e) => eprintln!("Error sending config update notification: {}", e.to_string())
        }
    }
}

#[derive(Clone)]
pub struct Data{
    data: Arc<ron::Value>,
    config_notify: Option<mpsc::Sender<()>>
}
impl Data {
    fn new(config_notify: mpsc::Sender<()>, value: ron::Value) -> Self {
        Self {
            data: Arc::new(value),
            config_notify: Some(config_notify)
        }
    }
    fn set_notification(&mut self, config_notify: mpsc::Sender<()>) {
        self.config_notify = Some(config_notify);
    }
}
impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.data.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = ron::Value::deserialize(deserializer)?;
        Ok(Self{
            data: Arc::new(value),
            config_notify: None,
        })
    }
}
impl Deref for Data {
    type Target = ron::Value;
    fn deref(&self) -> &Self::Target {
        &self.data
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
        match std::fs::write(&self.filepath, str_config) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to write {}: {}", self.filepath.to_string_lossy(), e.to_string())),
        }
    }
    pub fn register<S: AsRef<str>>(config: Arc<Config>, name: S) -> Data {
        // let mut components = &mut config.components;
        // let name = name.as_ref();
        // if components.contains_key(name) {
        //     let mut cmp = components.get_mut(name).unwrap();
        //     if let None = cmp.1 {
        //         cmp.set_config(config.clone());
        //     }
        //     cmp.clone()
        // } else {
        //     let value = Data::new(config.clone(), ron::Value::Unit);
        //     components.insert(name.to_string(), value.clone());
        //     value
        // }
    }
}