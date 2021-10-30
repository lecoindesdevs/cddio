use serenity::async_trait;
use std::sync::Arc;
use futures::lock::Mutex;

mod event;
mod framework;
pub mod command_parser;
pub mod components;

pub use event::EventListenerContainer as EventContainer;
pub use framework::{CDDFramework as Framework, FrameworkConfig};
pub use framework::{Context, Message};
pub use serenity::model::event::Event;

pub type ArcMut<T> = Arc<Mutex<T>>;

pub type ArcComponent = ArcMut<dyn Component>;

pub enum CommandMatch {
    Matched,
    NotMatched,
    Error(String)
}

#[async_trait]
pub trait Component: Sync + Send
{
    fn name(&self) -> &str;
    async fn command(&mut self, fw_config: &FrameworkConfig, ctx: &Context, msg: &Message) -> CommandMatch;
    async fn event(&mut self, ctx: &Context, evt: &Event) -> Result<(), String>;
    fn group_parser(&self) -> Option<&command_parser::Group> {
        None
    }
    fn load_config(&mut self, _config: ron::Value) {
        // Do nothing
    }
    fn get_config(&self) -> Option<&ron::Value> {
        None
    }
}

pub fn to_arc_mut<M>(mid: M) -> ArcMut<M> {
    Arc::new(Mutex::new(mid))
}
