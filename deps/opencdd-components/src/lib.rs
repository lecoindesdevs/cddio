pub mod declarative;
pub mod event;
pub mod container;
use std::sync::Arc;

pub use declarative::ComponentDeclarative;
pub use event::ComponentEvent;
pub use container::ComponentContainer;
use serenity::prelude::Mutex;

pub trait Component: ComponentDeclarative + ComponentEvent {}

pub type Components = Vec<Arc<Mutex<dyn Component>>>;
