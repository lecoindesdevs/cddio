
use serenity::async_trait;
use serenity::client::{Context, RawEventHandler};
pub use serenity::model::event::Event;

use super::ArcMiddleware;

#[derive(Default)]
pub struct EventListenerContainer {
    event_listeners: Vec<ArcMiddleware>,
}

impl EventListenerContainer {
    pub fn init() -> EventListenerContainer {
        EventListenerContainer::default()
    }
    pub fn add_middleware(&mut self, event_listener: ArcMiddleware) {
        self.event_listeners.push(event_listener);
    }
}

#[async_trait]
impl RawEventHandler for EventListenerContainer {
    async fn raw_event(&self, ctx: Context, evt: Event) {
        for mid in &self.event_listeners {
            let mut mid = mid.lock().await;
            if let Err(what) = mid.event(&ctx, &evt).await {
                println!("[{}] Module {} command error: {}\nEvent: {:?}\n\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
                    mid.name(),
                    what,
                    evt
                );
            }
        }
    }
}
