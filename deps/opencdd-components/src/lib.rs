pub mod declarative;
pub mod event;
use std::sync::Arc;

pub use declarative::ComponentDeclarative;
pub use event::ComponentEvent;
use serenity::prelude::Mutex;

pub trait Component: ComponentDeclarative + ComponentEvent {}

pub type Components = Vec<Arc<Mutex<dyn Component>>>;

pub struct ComponentContainer(Components);

impl ComponentContainer {
    pub fn new() -> ComponentContainer {
        ComponentContainer(Vec::new())
    }
    pub fn get_event_dispatcher(&self) -> event::ComponentEventDispatcher {
        event::ComponentEventDispatcher::new(self.0.clone())
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