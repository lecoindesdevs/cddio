use std::{sync::Arc, fmt::Display, str::FromStr, collections::HashMap};
use async_std::io::WriteExt;
use chrono::{Utc, DateTime};
use futures_locks::RwLock;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serenity::async_trait;
use tokio::sync::watch;

#[async_trait]
pub trait Callable: Send + Sync + Clone + 'static {
    type Data: Send+Sync;
    async fn call(&self, data: Arc<Self::Data>);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task<F> where 
    F: Callable
{
    until: i64,
    callable: F
}

pub type TaskID = u64;
pub trait Registry<F> 
where
    F: Callable
{
    fn register(&self, callable: Task<F>) -> Result<TaskID, String>;
    fn unregister(&self, id: TaskID) -> Result<(), String>;
}



#[derive(Debug)]
pub struct Tasks<F, Data> where 
    F: Callable<Data=Data> + Serialize + DeserializeOwned,
    Data: Send + Sync + Clone + 'static
{
    tasks: RwLock<HashMap<u32, Task<F>>>,
    task_counter: u32,
    path_file: std::path::PathBuf,
    waker: (watch::Sender<()>, watch::Receiver<()>),
    data: Arc<Data>
}

impl<F, Data> Tasks<F, Data> where
    F: Callable<Data=Data> + Serialize + DeserializeOwned,
    Data: Send + Sync + Clone + 'static
{
    pub async fn from_file(path_file: std::path::PathBuf, data: Data) -> Result<Self, String> {
        let mut res = Self {
            tasks: RwLock::new(HashMap::new()),
            path_file,
            waker: watch::channel(()),
            data: Arc::new(data),
            task_counter: 1
        };
        res.load().await?;
        Ok(res)
    }
    pub async fn add(&mut self, callable: F, until: DateTime<Utc>) {
        let task = Task{
            until: until.timestamp(), 
            callable: Arc::new(callable)
        };
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(self.task_counter, task.clone());
        }
        self.run((self.task_counter, task));
        self.task_counter += 1;
        self.save().await
            .or_else::<(), _>(|e| {
                println!("{}", e);
                Ok(())
            })
            .unwrap();
    }
    fn stop(&mut self) {
        self.waker.0.send(()).unwrap();
    }
    fn run(&self, task: (u32, Task<F>)) {
        use tokio::time::*;
        let tasks = RwLock::clone(&self.tasks);
        let mut waker = self.waker.1.clone();
        let data = Arc::clone(&self.data);
        tokio::spawn(async move {
            let duration = Duration::from_secs(i64::max(task.1.until - Utc::now().timestamp(), 0)  as _);
            match tokio::time::timeout(duration, waker.changed()).await {
                Err(_) => {
                    task.1.callable.call(data).await;
                    tasks.write().await.remove(&task.0);
                },
                Ok(_) => ()
            };
        });
        
    }
    async fn save(&self) -> Result<(), String> {
        use async_std::{fs::File};
        let data = ron::to_string(&*self.tasks.read().await)
            .map_err(|e| format!("Impossible de serialiser les taches: {}", e.to_string()))?;            
        let mut file = File::open(&self.path_file).await 
            .map_err(|e| format!("Impossible d'ouvrir le fichier {}: {}", self.path_file.display(), e.to_string()))?;
        file.write_all(data.as_bytes()).await
            .map_err(|e| format!("Impossible d'écrire dans le fichier {}: {}", self.path_file.display(), e.to_string()))
    }
    async fn load(&mut self) -> Result<(), String> {
        use async_std::fs;
        let content = fs::read_to_string(&self.path_file).await
            .map_err(|e| format!("Impossible de lire le fichier {}: {}", self.path_file.display(), e.to_string()))?;
        self.tasks = RwLock::new(ron::from_str(&content)
            .map_err(|e| format!("Impossible de lire le fichier: {}", e.to_string()))?);
        Ok(())
    }
}
impl<F, Data> Drop for Tasks<F, Data> where 
    F: Callable<Data=Data> + Serialize + DeserializeOwned,
    Data: Send + Sync + Clone + 'static
{
    fn drop(&mut self) {
        self.stop();
    }
}