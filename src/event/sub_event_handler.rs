use futures::lock::Mutex;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::event::Event;

/// This core trait for handling raw events
#[async_trait]
pub trait SubRawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    async fn raw_event(&mut self, ctx: &Mutex<Context>, evt: &Mutex<Event>);
}
