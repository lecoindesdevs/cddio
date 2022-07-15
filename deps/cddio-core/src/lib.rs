//! # Core library of CDDIO
//! 
//! This library provides the core functionality of the CDDIO library.
//! 
//! ## Components system
//! 
//! The library is designed to work with components. Each component manage their own data
//! and can be used to interact with the client or other components.
//! 
//! Each component must implement the [`Component`] trait to handle event and application command.
//! The [`Component`] trait is composed of two traits: 
//! - [`ComponentEvent`] which manage Discord gateway events.
//! - [`ComponentDeclarative`] which manage applications 
//!     commands declatation (groups, command names, arguments, description...)
//! 
//! ## Simplify serenity
//! 
//! The crate [`serenity`] is a brut implementation of the Discord API in pure Rust. 
//! While [`serenity`] implement the Discord API very well and manage low level functionnalities, 
//! it does not provide facilities and shortcuts to easily interact with the Discord API.
//! 
//! This library aims to be overlay to the [`serenity`] crate to answer to this problem.
//! 
//! ## Macros
//! 
//! On top of this crate, you can take a look at the [`cddio-macros`] crate which provides 
//! even more facilities to describes components (like events and macros).
//! 
//! [`cddio-macros`]: ../cddio_macros/index.html

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

pub trait Component: ComponentDeclarative + ComponentEvent {}
pub type Components = Vec<Arc<dyn Component>>;
