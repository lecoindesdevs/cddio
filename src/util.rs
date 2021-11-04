//! Fonctions utiles pour l'ensemble du projet

use std::{ops::{Deref, DerefMut}, sync::Arc};

use futures_locks::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[doc = "Macro pour créer un VecDeque de la même manière que vec!"]
#[doc(alias = "std::collections::VecDeque")]
#[macro_export]
macro_rules! vdq {
    ($($args:expr),*) => {
        {
            let mut v = std::collections::VecDeque::new();
            $(v.push_back($args);)*
            v
        }
    };
}
#[derive()]
pub struct ArcRw<T>(Arc<RwLock<T>>);
pub type ArcRwBox<T> = ArcRw<Box<T>>;

impl<T> ArcRw<T> {
    pub fn new(v: T) -> Self {
        ArcRw(Arc::new(RwLock::new(v)))
    }
    pub async fn read(&self) -> RwLockReadGuard<T> {
        self.0.read().await
    }
    pub async fn write(&self) -> RwLockWriteGuard<T> {
        self.0.write().await
    }
}