use serenity::async_trait;
use std::sync::Arc;
use futures::lock::Mutex;

mod event;
mod framework;
pub mod components;

pub use event::EventListenerContainer as EventContainer;
pub use framework::{CDDFramework as Framework, FrameworkConfig};

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
    async fn command(&mut self, fw_config: &framework::FrameworkConfig, ctx: &framework::Context, msg: &framework::Message) -> CommandMatch;
    async fn event(&mut self, ctx: &framework::Context, evt: &event::Event) -> Result<(), String>;
}

pub fn to_arc_mut<M>(mid: M) -> ArcMut<M> {
    Arc::new(Mutex::new(mid))
}
