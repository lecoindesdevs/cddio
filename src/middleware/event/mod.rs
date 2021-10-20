mod bot_start;

use std::sync::Arc;
use futures::lock::Mutex;
use serenity::async_trait;
use serenity::client::{Context, RawEventHandler};
use serenity::model::event::Event;

/// This core trait for handling raw events
#[async_trait]
pub trait SubRawEventHandler: Send + Sync {
    /// Dispatched when any event occurs
    #[inline]
    async fn raw_event(&mut self, ctx: &Mutex<Context>, evt: &Mutex<Event>)
    {}
}

pub type ArcEvent = Arc<Mutex<dyn SubRawEventHandler>>;


#[derive(Default)]
pub struct EventListenerContainer {
    event_listeners: Vec<ArcEvent>,
}

impl EventListenerContainer {
    pub fn init() -> EventListenerContainer {
        EventListenerContainer::default()
    }
    pub fn add_event_listener(&mut self, event_listener: ArcEvent) {
        self.event_listeners.push(event_listener);
    }
}

#[async_trait]
impl RawEventHandler for EventListenerContainer {
    async fn raw_event(&self, ctx: Context, evt: Event) {
        let ctx = Mutex::new(ctx);
        let evt = Mutex::new(evt);
        for middleware in &self.event_listeners {
            let mut middleware = middleware.lock().await;
            middleware.raw_event(&ctx, &evt).await
        }
    }
}
