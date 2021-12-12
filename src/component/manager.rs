use super::ArcComponent;
use crate::util::ArcRw;

pub struct Manager {
    components: Vec<ArcComponent>,
}

impl Manager {
    pub fn new () -> Self {
        Manager {
            components: Vec::new(),
        }
    }
    pub fn add_component(&mut self, cmp_arc: ArcComponent) -> &mut Self {
        self.components.push(cmp_arc);
        self
    }
    pub fn get_components(&self) -> &Vec<ArcComponent> {
        &self.components
    }
}

pub type ArcManager = ArcRw<Manager>;
