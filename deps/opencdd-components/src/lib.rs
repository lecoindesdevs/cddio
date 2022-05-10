pub mod declarative;
pub mod event;
pub use declarative::ComponentDeclarative;
pub use event::ComponentEvent;

pub trait Component: ComponentDeclarative + ComponentEvent {}

