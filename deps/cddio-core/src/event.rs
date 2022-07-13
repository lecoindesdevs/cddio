use serenity::{model::event::Event, client::Context, async_trait};
pub use serenity::prelude::RawEventHandler;
use crate::Components;

/// # The component event trait. 
/// 
/// Every component must implement this trait to receive gateway event from ComponentEventDispatcher.
#[async_trait]
pub trait ComponentEvent: Sync + Send{
    async fn event(&self, ctx: &Context, event: &Event);
}

/// # The component event dispatcher.
/// 
/// This dispatcher is responsible for dispatching events to the components.
/// Add it to the client to receive events.
/// 
/// See [`serenity::client::ClientBuilder::raw_event_handler()`] for more information.
pub struct ComponentEventDispatcher {
    components: Components
}

impl ComponentEventDispatcher {
    pub(crate) fn new(components: Components) -> Self { 
        Self { components } 
    }
}

#[async_trait]
impl RawEventHandler for ComponentEventDispatcher {
    async fn raw_event(&self, ctx: Context, ev: Event) {
        for comp in &self.components {
            comp.event(&ctx, &ev).await
        }
    }
}