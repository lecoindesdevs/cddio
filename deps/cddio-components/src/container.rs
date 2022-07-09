use std::sync::Arc;
use futures_locks::RwLock;
use crate::{Components, event::ComponentEventDispatcher, Component};

#[derive(Clone)]
pub struct ComponentContainer(Components);
pub type RefContainer = RwLock<ComponentContainer>;

impl ComponentContainer {
    pub fn new() -> ComponentContainer {
        ComponentContainer(Vec::new())
    }
    pub fn get_event_dispatcher(&self) -> ComponentEventDispatcher {
        ComponentEventDispatcher::new(self.0.clone())
    }
    pub fn add_component<T: 'static + Component>(&mut self, comp: T) -> Arc<T> {
        let arc = Arc::new(comp);
        let v = Arc::clone(&arc);
        self.0.push (v);
        arc
    }
}
impl AsRef<Components> for ComponentContainer {
    fn as_ref(&self) -> &Components {
        &self.0
    }
}
impl AsMut<Components> for ComponentContainer {
    fn as_mut(&mut self) -> &mut Components {
        &mut self.0
    }
}