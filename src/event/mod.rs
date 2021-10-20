mod sub_event_handler;
mod bot_start;

use futures::lock::Mutex;
use serenity::{async_trait};
use serenity::client::{Context, RawEventHandler};
use serenity::model::event::Event;

use sub_event_handler::SubRawEventHandler;

struct EventListener {
    name: String,
    listener: Mutex<Box<dyn SubRawEventHandler>>,
}

#[derive(Default)]
pub struct EventListenerContainer {
    event_listeners: Vec<EventListener>,
}

impl EventListenerContainer {
    pub fn init() -> EventListenerContainer {
        let mut evts = EventListenerContainer::default();
        evts.register_event_listener("bot_start", Box::new(bot_start::BotStart));
        evts
    }
    fn register_event_listener(&mut self, name: &str, event_listener: Box<dyn SubRawEventHandler>) {
        self.event_listeners.push(EventListener {
            name: name.to_string(),
            listener: Mutex::new(event_listener),
        });
    }
}

#[async_trait]
impl RawEventHandler for EventListenerContainer {
    async fn raw_event(&self, ctx: Context, evt: Event) {
        let ctx = Mutex::new(ctx);
        let evt = Mutex::new(evt);
        for middleware in &self.event_listeners {
            let mut middleware = middleware.listener.lock().await;
            middleware.as_mut().raw_event(&ctx, &evt).await
        }
    }
}
