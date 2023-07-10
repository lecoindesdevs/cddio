use std::{
    path::{PathBuf, Path}, 
    ops::{
        DerefMut, 
        Deref
    }
};

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Serde(serde_json::Error),
}
#[derive(Debug)]
pub struct Data<T>(RwLock<T>, PathBuf);

impl<T> Data<T> {
    fn filename(stem: &str) -> PathBuf {
        PathBuf::from(format!("{}.json", stem))
    }
    pub fn new(stem: &str, data: T) -> Self {
        Self(RwLock::new(data), Self::filename(stem))
    }
    pub fn exists(stem: &str) -> bool {
        Self::filename(stem).exists()
    }
}
impl<T: DeserializeOwned> Data<T> {
    pub fn from_file(stem: &str) -> Result<Self, Error> {
        let filepath = Self::filename(stem);
        let file_content = std::fs::read_to_string(&filepath).map_err(Error::Io)?;
        let data: T = serde_json::from_str(&file_content).map_err(Error::Serde)?;
        Ok(Self(RwLock::new(data), filepath))
    }
}
impl<T: DeserializeOwned + Default> Data<T> {
    pub fn from_file_or_default(stem: &str) -> Result<Self, Error> {
        let filepath = Self::filename(stem);
        let data = if filepath.exists() {
            let file_content = std::fs::read_to_string(&filepath).map_err(Error::Io)?;
            serde_json::from_str(&file_content).map_err(Error::Serde)?
        } else {
            T::default()
        };
        Ok(Self(RwLock::new(data), filepath))
    }
}
impl<T: Serialize> Data<T> {
    pub async fn read(&self) -> DataGuard<'_, T> {
        DataGuard::Read(self.0.read().await)
    }
    pub async fn write(&self) -> DataGuard<'_, T> {
        DataGuard::Write(self.0.write().await, self.1.as_path())
    }
}

pub enum DataGuard<'a, T> 
where T: Serialize
{
    Read(RwLockReadGuard<'a, T>),
    Write(RwLockWriteGuard<'a, T>, &'a Path)
}

impl<'a, T: Serialize> Deref for DataGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            DataGuard::Read(ref guard) => guard.deref(),
            DataGuard::Write(ref guard, _) => guard.deref()
        }
    }
}
impl<'a, T: Serialize> DerefMut for DataGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DataGuard::Read(_) => panic!("Read guard cannot be mutated"),
            DataGuard::Write(ref mut guard, _) => guard.deref_mut()
        }
    }
}
impl<'a, T: Serialize> Drop for DataGuard<'a, T> 
{
    fn drop(&mut self) {
        if let DataGuard::Write(data, path) = self {
            let value = RwLockWriteGuard::deref(data);
            let value = serde_json::to_string(value).expect("Unable to serialize data");
            std::fs::write(path, value).expect("Unable to write data");
        }
    }
}