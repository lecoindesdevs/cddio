use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub prefix: char,
    pub permissions: u64,
}

impl Config {
    pub fn read_file<P: AsRef<std::path::Path>>(filepath: P) -> Result<Self, String> {
        let str_config = match std::fs::read_to_string(filepath.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(format!("Unable to read file {}: {}", filepath.as_ref().to_string_lossy(), e.to_string())),
        };
        match serde_json::from_str(&str_config) {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Unable to parse config.json: {}", e.to_string())),
        }
    }
}