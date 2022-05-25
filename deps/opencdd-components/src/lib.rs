pub mod declarative;
pub mod event;
pub mod container;
pub mod embed;
pub mod message;
use std::sync::Arc;

pub use declarative::ComponentDeclarative;
pub use event::ComponentEvent;
pub use container::ComponentContainer;
pub use embed::ApplicationCommandEmbed;
use serenity::prelude::Mutex;

pub trait Component: ComponentDeclarative + ComponentEvent {}
pub type Components = Vec<Arc<dyn Component>>;
