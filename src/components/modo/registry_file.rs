use std::collections::HashMap;
use crate::{log_error, log_warn, log_info};

use async_std::io::WriteExt;
use futures_locks::RwLock;

use serenity::{
    async_trait
};

use super::sanction::Sanction;
use super::task;

pub struct RegistryFile {
    path_file: std::path::PathBuf,
    tasks: RwLock<HashMap<task::TaskID, task::Task<Sanction>>>,
    task_counter: RwLock<task::TaskID>
}

impl RegistryFile {
    pub async fn from_file(path_file: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let res = Self {
            path_file: path_file.as_ref().to_path_buf(),
            tasks: RwLock::new(HashMap::new()),
            task_counter: RwLock::new(1)
        };
        res.load().await?;
        Ok(res)
    }
    async fn save(&self) -> Result<(), String> {
        let log_error = |msg, e| {
            let e = format!("modo::RegistryFile::save: {}: {}", msg, e);
            log_error!("{}", e);
            e
        };
        let mut file = async_std::fs::File::create(&self.path_file).await
            .map_err(|e| log_error(format!("Unable to open/create file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;

        let tasks = self.tasks.read().await;
        let data = serde_json::to_string(&*tasks)
            .map_err(|e| log_error(format!("Unable to serialize tasks"), e.to_string()))?;
        file.write_all(data.as_bytes()).await
            .map_err(|e| log_error(format!("Unable to open/create file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;
        Ok(())
    }
    async fn load(&self) -> Result<(), String> {
        let log_error = |msg, e| {
            let e = format!("modo::RegistryFile::load: {}: {}", msg, e);
            log_error!("{}", e);
            e
        };
        if self.path_file.exists() {
            let data = std::fs::read_to_string(&self.path_file)
                .map_err(|e| log_error(format!("Unable to read file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;
            let tasks: HashMap<_,_> = serde_json::from_str(&data)
                .map_err(|e| log_error(format!("Unable to parse tasks"), e.to_string()))?;
            let highest_id = tasks.iter().map(|(id, _)| *id).max().unwrap_or(0);
            *self.tasks.write().await = tasks;
            *self.task_counter.write().await = highest_id + 1;
        }
        Ok(())
    }
}
#[async_trait]
impl task::Registry for RegistryFile {
    type Data = Sanction;
    async fn register(&mut self, task: task::Task<Self::Data>) -> Result<task::TaskID, String> {
        let id = self.task_counter.read().await.clone();
        self.tasks.write().await.insert(id, task);
        *self.task_counter.write().await += 1;
        match self.save().await {
            Ok(_) => Ok(id),
            Err(e) => Err(e)
        }
    }
    async fn unregister(&mut self, id: task::TaskID) -> Result<(), String> {
        self.tasks.write().await.remove(&id);
        match self.save().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    async fn get(&self, id: task::TaskID) -> Option<task::Task<Self::Data>> {
        self.tasks.read().await.iter().find(|(vid, _)| vid == &&id).map(|(_, task)| task.clone())
    }

    async fn get_all(&self) -> Vec<(task::TaskID, task::Task<Self::Data>)> {
        self.tasks.read().await.iter().map(|v| (*v.0, v.1.clone())).collect()
    }

    async fn find_one<F>(&self, f: F) -> Option<(task::TaskID, task::Task<Self::Data>)> where
        F: Fn(&task::Task<Self::Data>) -> bool + Send
    {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .find(|(_, task)| f(task))
            .map(|(id, task)| (*id, task.clone()))
    }
    async fn find_all<F>(&self, f: F) -> Vec<(task::TaskID, task::Task<Self::Data>)> where
        F: Fn(&task::Task<Self::Data>) -> bool + Send
    {
        self.tasks.read().await
            .iter()
            .filter(|(_, task)| f(task))
            .map(|v| (*v.0, v.1.clone()))
            .collect()
    }
}