use std::sync::{Arc};
use futures_locks::RwLock;
use serenity::async_trait;

use super::ArcComponent;

pub struct Manager {
    components: Vec<ArcComponent>,
}

impl Manager {
    pub fn new () -> Self {
        Manager {
            components: Vec::new(),
        }
    }
    pub fn add_component(&mut self, cmp_arc: ArcComponent) {
        self.components.push(cmp_arc);
    }
    pub fn get_components(&self) -> &Vec<ArcComponent> {
        &self.components
    }
}

pub type ArcManager = Arc<RwLock<Manager>>;
