use std::{
    path::{PathBuf, Path}, 
    ops::{
        DerefMut, 
        Deref
    }, sync::Arc
};

use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use crate::db::model::app_data;

#[derive(Debug)]
pub enum Error {
    SeaOrm(sea_orm::error::DbErr),
    Serde(serde_json::Error),
}
#[derive(Debug)]
pub struct Data<T: Serialize + DeserializeOwned> {
    pub data: RwLock<T>, 
    pub db: Arc<DatabaseConnection>,
    pub name: String
};

impl<T: Serialize + DeserializeOwned> Data<T> {
    pub async fn new<S: Into<String>>(db: Arc<DatabaseConnection>, name: S, data: T) -> Self {
        let v = Self {
            data: RwLock::new(data),
            db,
            name: name.into(),
        };
        {
            // Will trigger the save in database
            v.write().await;
        }
        v
    }
    pub fn from_file(name: &str) -> Result<Self, Error> {
        let filepath = Self::filename(name);
        let file_content = std::fs::read_to_string(&filepath).map_err(Error::Io)?;
        let data: T = serde_json::from_str(&file_content).map_err(Error::Serde)?;
        Ok(Self(RwLock::new(data), filepath))
    }
    pub async fn exists(name: &str, db: &DatabaseConnection) -> Result<bool, Error> {
        Ok(app_data::Entity::find_by_id(name)
            .one(db).await
            .map_err(Error::SeaOrm)?
            .is_some())
    }
    pub async fn read(&self) -> DataGuard<'_, T> {
        DataGuard::Read(self.data.read().await)
    }
    pub async fn write(&self) -> DataGuard<'_, T> {
        DataGuard::Write(self.data.write().await, self.1.as_path())
    }
}

impl<T: Serialize + DeserializeOwned + Default> Data<T> {
    pub fn from_file_or_default(name: &str) -> Result<Self, Error> {
        let filepath = Self::filename(name);
        let data = if filepath.exists() {
            let file_content = std::fs::read_to_string(&filepath).map_err(Error::Io)?;
            serde_json::from_str(&file_content).map_err(Error::Serde)?
        } else {
            T::default()
        };
        Ok(Self(RwLock::new(data), filepath))
    }
}

pub enum DataGuard<'a, T: Serialize> {
    Read(RwLockReadGuard<'a, T>),
    Write(RwLockWriteGuard<'a, T>, DatabaseConnection, &'a str)
}

impl<'a, T: Serialize> Deref for DataGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            DataGuard::Read(ref guard) => guard.deref(),
            DataGuard::Write(ref guard, ..) => guard.deref()
        }
    }
}
impl<'a, T: Serialize> DerefMut for DataGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DataGuard::Read(_) => panic!("Read guard cannot be mutated"),
            DataGuard::Write(ref mut guard, ..) => guard.deref_mut()
        }
    }
}
impl<'a, T: Serialize> Drop for DataGuard<'a, T> 
{
    fn drop(&mut self) {
        if let DataGuard::Write(data, db, name) = self {
            let value = serde_json::to_string(data.deref()).expect("Unable to serialize data");
            let item = match app_data::Entity::find_by_id(name).one(db).await.expect("Unable to get data from database") {
                Some(v) => v.,
                None => todo!(),
            }
        }
    }
}