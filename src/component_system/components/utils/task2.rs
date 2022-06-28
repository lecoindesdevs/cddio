use std::{time::Duration, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use futures_locks::Mutex;
use serde::{Deserialize, Serialize};
use serenity::async_trait;

#[async_trait]
pub trait DataFunc: Send + Sync + 'static {
    async fn run(&self) -> Result<(), String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task<D: DataFunc + Clone> 
{
    until: i64,
    data: D
}

pub type TaskID = u64;

#[derive(Clone)]
struct RegistryTask<D> where
    D: DataFunc + Clone
{
    id: TaskID,
    task: Task<D>
}

#[async_trait]
pub trait Registry
{
    type Data: DataFunc + Clone;
    async fn register(&mut self, task: Task<Self::Data>) -> Result<TaskID, String>;
    async fn unregister(&mut self, id: TaskID) -> Result<(), String>;
    async fn get(&self, id: TaskID) -> Option<Task<Self::Data>>;
    async fn get_all(&self) -> Vec<RegistryTask<Self::Data>>;
}

type Tasks<R: Registry> = Arc<Mutex<R>>;

pub struct TaskManager<D, R> where
    D: DataFunc + Clone,
    R: Registry<Data = D> + Send + 'static
{
    tasks: Tasks<R>,
    task_handles: HashMap<TaskID, tokio::task::JoinHandle<()>>
}

impl<D, R> TaskManager<D, R> where
    D: DataFunc + Clone,
    R: Registry<Data = D> + Send + 'static
{
    pub fn new(registry: R) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(registry)),
            task_handles: HashMap::new()
        }
    }
    pub async fn add(&mut self, data: D, until: DateTime<Utc>) -> Result<TaskID, String> {
        let mut tasks = self.tasks.lock().await;
        let id = tasks.register(Task {
            until: until.timestamp(),
            data: data.clone()
        }).await?;
        let tasks = Arc::clone(&self.tasks);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs((until.timestamp() - Utc::now().timestamp()) as _ )).await;
            data.run().await;
            Self::remove_from_registry(&tasks, id).await;
        });
        self.task_handles.insert(id, handle);
        Ok(id)
    }
    pub async fn remove(&mut self, id: TaskID) -> Result<(), String> {
        match self.task_handles.get(&id){
            Some(handle) => {
                handle.abort();
                self.task_handles.remove(&id);
                Self::remove_from_registry(&self.tasks, id).await;
                Ok(())
            },
            None => Err("Task not found".to_string())
        }
    }
    async fn remove_from_registry(tasks: &Tasks<R>, id: TaskID) -> Result<(), String> {
        let mut tasks = tasks.lock().await;
        tasks.unregister(id).await;
        Ok(())
    }
    pub async fn get(&self, id: TaskID) -> Option<Task<D>> {
        let tasks = self.tasks.lock().await;
        tasks.get(id).await
    }
}

impl<D, R> Drop for TaskManager<D, R> where
    D: DataFunc + Clone,
    R: Registry<Data = D> + Send + 'static
{
    fn drop(&mut self) {
        for (_, task) in self.task_handles.iter() {
            task.abort();
        }
    }
}