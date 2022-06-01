use std::io::Write;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::sync::watch;

// trait Callable<'a>: FnOnce() + Serialize + Deserialize<'a> {}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Task<F> where 
    F: FnOnce()// + Send + Clone + 'static
{
    until: i64,
    callable: F
}

impl<F> Task<F> where
    F: FnOnce() + Send + Clone + 'static
{
    pub fn new(until: DateTime<Utc>, callable: F) -> Self {
        Self {
            until: until.timestamp(),
            callable
        }
    }
    pub fn run(&self, waker: watch::Receiver<()>) {
        use tokio::time::*;
        let duration = Duration::from_secs((self.until - Utc::now().timestamp()) as _);
        if duration < Duration::from_secs(0) {
            (self.callable)();
            return;
        }
        let callable = F::clone(&self.callable);
        
        tokio::spawn(Self::wait(duration, waker, callable));
    }
    async fn wait(duration: tokio::time::Duration, mut waker: watch::Receiver<()>, callable: F) {
        match tokio::time::timeout(duration, waker.changed()).await {
            Err(_) => callable(),
            Ok(_) => ()
        };
    }
}
#[derive(Debug)]
struct Tasks<F> where 
    F: FnOnce() + Send + Clone + Serialize + DeserializeOwned + 'static
{
    tasks: Vec<Task<F>>,
    path_file: std::path::PathBuf,
    waker: (watch::Sender<()>, watch::Receiver<()>)
}

impl<F> Tasks<F> where
    F: FnOnce() + Send + Clone + Serialize + DeserializeOwned + 'static
{
    pub fn new(path_file: std::path::PathBuf) -> Self {
        let res = Self {
            tasks: Vec::new(),
            path_file,
            waker: watch::channel(())
        };
        res.load();
        res
    }
    pub fn add(&mut self, callable: F, until: DateTime<Utc>) {
        let task = Task::new(until, callable);
        task.run(self.waker.1.clone());
        self.tasks.push(task);
        self.save();
    }
    pub fn stop(&mut self) {
        self.waker.0.send(()).unwrap();
    }
    pub fn save(&self) -> Result<(), String> {
        let data = match ron::to_string(&self.tasks) {
            Ok(data) => data,
            Err(e) => return Err(format!("Impossible de serialiser les taches: {}", e.to_string())),
        };
        let mut file = match std::fs::File::create(&self.path_file) {
            Ok(file) => file,
            Err(e) => return Err(format!("Impossible d'ouvrir le fichier {}: {}", self.path_file.display(), e.to_string())),
        };
        match file.write(data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Impossible d'écrire dans le fichier {}: {}", self.path_file.display(), e.to_string())),
        }
    }
    pub fn load(&mut self) -> Result<(), String> {
        let content = match std::fs::read_to_string(&self.path_file) {
            Ok(file) => file,
            Err(e) => return Err(format!("Impossible de lire le fichier {}: {}", self.path_file.display(), e.to_string()))
        };
        self.tasks = match ron::from_str(&content) {
            Ok(tasks) => tasks,
            Err(e) => return Err(format!("Impossible de lire le fichier: {}", e.to_string()))
        };
        Ok(())
    }
}
impl<F> Drop for Tasks<F> where
    F: FnOnce() + Send + Clone + Serialize + DeserializeOwned + 'static
{
    fn drop(&mut self) {
        self.stop();
    }
}