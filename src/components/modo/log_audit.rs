use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;

pub struct Log {
    path: PathBuf,
}

#[derive(Serialize)]
struct LogEntry<D> {
    datetime: DateTime<Utc>,
    data: D,
}

impl Log {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
        }
    }
    pub async fn push<D: Serialize>(&self, data: &D) -> Result<(), String> {
        use async_std::fs::OpenOptions;
        use async_std::prelude::*;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path).await
            .or_else(|e| Err(format!("modo: Impossible d'ouvrir le fichier de log: {}", e.to_string())))?;
        let log_str = serde_json::to_string(&LogEntry{
            datetime: Utc::now(),
            data: data,
        })
            .or_else(|e| Err(format!("modo: Impossible de convertir la sanction en RON: {}", e.to_string())))?;
        file.write(log_str.as_bytes()).await
            .or_else(|e| Err(format!("modo: Impossible d'écrire le fichier de log: {}", e.to_string())))?;
        file.write(b"\n").await
            .or_else(|e| Err(format!("modo: Impossible d'écrire dans le fichier de log: {}", e.to_string())))?;
        Ok(())
    }
}