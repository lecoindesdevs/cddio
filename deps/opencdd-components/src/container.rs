use crate::{Components, event::ComponentEventDispatcher};

pub struct ComponentContainer(Components);

impl ComponentContainer {
    pub fn new() -> ComponentContainer {
        ComponentContainer(Vec::new())
    }
    pub fn get_event_dispatcher(&self) -> ComponentEventDispatcher {
        ComponentEventDispatcher::new(self.0.clone())
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