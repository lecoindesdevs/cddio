use serenity::async_trait;
use std::sync::Arc;
use futures::lock::Mutex;

mod event;
mod framework;

pub use event::EventListenerContainer as EventContainer;
pub use framework::{CDDFramework as Framework, FrameworkConfig};


pub type ArcComponent = Arc<Mutex<dyn Component>>;

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

pub fn to_arc<M: Component + 'static>(mid: M) -> ArcComponent {
    Arc::new(Mutex::new(mid))
}

mod bot_start;
pub use bot_start::BotStart;