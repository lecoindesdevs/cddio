//! Task runner for the bot. 
//! 
//! Used to create and feed a tasks manager. 
//! Every task is executed in background after a bunch of time.

use std::{time::Duration, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc, TimeZone};
use futures_locks::Mutex;
use crate::{log_error, log_info};
use serde::{Deserialize, Serialize};
use serenity::async_trait;

#[async_trait]
pub trait DataFunc: Send + Sync + 'static {
    type Persistent: Send + Sync + 'static;
    async fn run(&self, persistent: &Self::Persistent) -> Result<(), String>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task<D: DataFunc + Clone> 
{
    pub until: i64,
    pub data: D
}

pub type TaskID = u64;

#[async_trait]
pub trait Registry
{
    type Data: DataFunc + Clone;
    async fn register(&mut self, task: Task<Self::Data>) -> Result<TaskID, String>;
    async fn unregister(&mut self, id: TaskID) -> Result<(), String>;
    async fn get(&self, id: TaskID) -> Option<Task<Self::Data>>;
    async fn get_all(&self) -> Vec<(TaskID, Task<Self::Data>)>;
    async fn find_one<F>(&self, f: F) -> Option<(TaskID, Task<Self::Data>)> where
        F: Fn(&Task<Self::Data>) -> bool + Send;
    async fn find_all<F>(&self, f: F) -> Vec<(TaskID, Task<Self::Data>)> where
        F: Fn(&Task<Self::Data>) -> bool + Send;
}

type Tasks<R> = Arc<Mutex<R>>;

pub struct TaskManager<D, R, P> where
    D: DataFunc<Persistent = P> + Clone,
    R: Registry<Data = D> + Send + 'static,
    P: Send + Sync + 'static
{
    tasks: Tasks<R>,
    task_handles: HashMap<TaskID, tokio::task::JoinHandle<()>>,
    persistent: Arc<P>
}

impl<D, R, P> TaskManager<D, R, P> where
    D: DataFunc<Persistent = P> + Clone + std::fmt::Debug,
    R: Registry<Data = D> + Send + 'static,
    P: Send + Sync + 'static
{
    pub fn new(registry: R, persistent_data: P) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(registry)),
            task_handles: HashMap::new(),
            persistent: Arc::new(persistent_data)
        }
    }
    pub async fn init(&mut self) {
        let tasks = self.tasks.lock().await.get_all().await;
        log_info!("Initializing {} tasks", tasks.len());
        for (task_id, task) in tasks {
            log_info!("Initializing task {}. Data: {:?}", task_id, task);
            let handle = self.spawn_task(task_id, task.data, Utc.timestamp(task.until, 0));
            log_info!("Task {} initialized", task_id);
            self.task_handles.insert(task_id, handle);
        }
    }
    pub async fn add(&mut self, data: D, until: DateTime<Utc>) -> Result<TaskID, String> {
        let mut tasks = self.tasks.lock().await;
        let id = tasks.register(Task {
            until: until.timestamp(),
            data: data.clone()
        }).await?;
        let handle = self.spawn_task(id, data, until);
        self.task_handles.insert(id, handle);
        Ok(id)
    }
    fn spawn_task(&self, id: TaskID, data: D, until: DateTime<Utc>) -> tokio::task::JoinHandle<()> {
        let tasks = Arc::clone(&self.tasks);
        let persistent = Arc::clone(&self.persistent);
        tokio::spawn(async move {
            log_info!("Task {}: Spawning", id);
            let seconds = until.timestamp() - Utc::now().timestamp();
            if seconds > 0 {
                let duration = Duration::from_secs(seconds as _ );
                log_info!("Task {}: Sleeping for {} seconds", id, seconds);
                tokio::time::sleep(duration).await;
            }
            log_info!("Task {}: Running", id);
            if let Err(e) = data.run(&*persistent).await {
                log_error!("Task {} failed: {}", id, e);
                return;
            }
            if let Err(e) = Self::remove_from_registry(&tasks, id).await {
                log_error!("Task {} failed to remove from registry: {}", id, e);
                return;
            }
            log_info!("Task {}: Finished", id);
        })
    }
    pub async fn remove(&mut self, id: TaskID) -> Result<(), String> {
        match self.task_handles.get(&id){
            Some(handle) => {
                handle.abort();
                self.task_handles.remove(&id);
                if let Err(e) = Self::remove_from_registry(&self.tasks, id).await {
                    log_error!("Task {} failed to remove from registry: {}", id, e);
                }
                Ok(())
            },
            None => Err("Task not found".to_string())
        }
    }
    async fn remove_from_registry(tasks: &Tasks<R>, id: TaskID) -> Result<(), String> {
        let mut registry = tasks.lock().await;
        registry.unregister(id).await
    }
    pub async fn get(&self, id: TaskID) -> Option<Task<D>> {
        let tasks = self.tasks.lock().await;
        tasks.get(id).await
    }
    pub fn registry(&self) -> Arc<Mutex<R>> {
        Arc::clone(&self.tasks)
    }
    pub fn reset_persistent(&mut self, data: P) {
        self.persistent = Arc::new(data);
    }
}

impl<D, R, P> Drop for TaskManager<D, R, P> where
    D: DataFunc<Persistent = P> + Clone,
    R: Registry<Data = D> + Send + 'static, 
    P: Send + Sync + 'static
{
    fn drop(&mut self) {
        for (_, task) in self.task_handles.iter() {
            task.abort();
        }
    }
}