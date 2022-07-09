use serenity::{model::event::Event, client::Context, async_trait};
pub use serenity::prelude::RawEventHandler;
use crate::Components;

#[async_trait]
pub trait ComponentEvent: Sync + Send{
    async fn event(&self, ctx: &Context, event: &Event);
}

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