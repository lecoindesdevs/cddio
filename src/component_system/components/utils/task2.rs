use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::async_trait;

#[derive(Serialize, Deserialize, Clone)]
pub struct Task<D> 
{
    until: i64,
    data: D
}

pub type TaskID = u64;

#[derive(Serialize, Deserialize, Clone)]
pub struct RegistryTask<D>
{
    pub id: TaskID,
    pub task: Task<D>
}

#[async_trait]
pub trait Registry<D> {
    async fn register(&self, task: Task<D>) -> Result<TaskID, String>;
    async fn unregister(&self, id: TaskID) -> Result<(), String>;
    async fn get(&self, id: TaskID) -> Option<Task<D>>;
    async fn get_all(&self) -> Vec<RegistryTask<D>>;
}

pub struct TaskManager<D, R> where
    R: Registry<D>
{
    task_func: Box<dyn Fn(D) -> () + Send + Sync>,
    registry: R,
}

impl<D, R> TaskManager<D, R> where
    R: Registry<D>
{
    pub fn new(task_func: impl Fn(D) -> () + Send + Sync + 'static, registry: R) -> Self {
        Self {
            task_func: Box::new(task_func),
            registry
        }
    }
    pub async fn add(&self, data: D, until: DateTime<Utc>) -> Result<TaskID, String> {
        self.registry.register(Task { until: until.timestamp(), data }).await
    }
    pub async fn remove(&self, id: TaskID) -> Result<(), String> {
        self.registry.unregister(id).await
    }
    pub async fn get(&self, id: TaskID) -> Option<Task<D>> {
        self.registry.get(id).await
    }
    pub fn reset_func(&mut self, task_func: impl Fn(D) -> () + Send + Sync + 'static) {
        self.task_func = Box::new(task_func);
    }
    pub async fn run(&self) {
        loop {
            let tasks = self.registry.get_all().await;
            if tasks.len() == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            let now = Utc::now();
            for RegistryTask{id: task_id, task} in tasks {
                if task.until < now.timestamp() {
                    self.task_func.as_ref()(task.data);
                    self.remove(task_id).await;
                }
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}