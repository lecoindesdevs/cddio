use serenity::framework::{StandardFramework, standard::{Command, CommandGroup}};
use std::sync::Arc;
use futures::lock::Mutex;

mod event;
mod commands;

pub use event::EventListenerContainer as EventContainer;
pub use event::SubRawEventHandler as SubEvent;
pub use event::ArcEvent;

pub type ArcMiddleware = Arc<Mutex<dyn Middleware>>;


pub trait Middleware: SubEvent
{
    fn name(&self) -> &str;
    fn command_group<'a>(&'a self) -> Option<&'a CommandGroup>;
}


mod bot_start;
pub use bot_start::BotStart;