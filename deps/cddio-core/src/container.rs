use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Components, event::ComponentEventDispatcher, Component};

/// # The component container
/// 
/// The component container stores components to dispatch them into the client or other components.
#[derive(Clone)]
pub struct ComponentContainer(Components);
pub type RefContainer = Arc<RwLock<ComponentContainer>>;

impl ComponentContainer {
    pub fn new() -> ComponentContainer {
        ComponentContainer(Vec::new())
    }
    /// Create a [`ComponentEventDispatcher`] from the components in the container.
    /// Note that if new components are added to the container afterward, the dispatcher will not included these.
    pub fn get_event_dispatcher(&self) -> ComponentEventDispatcher {
        ComponentEventDispatcher::new(self.0.clone())
    }
    /// Add a component to the container. 
    /// The component is embedded to an Arc pointer to be async compatible.
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