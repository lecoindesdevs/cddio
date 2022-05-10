use serenity::{model::event::Event, client::{Context, RawEventHandler}, async_trait};

use crate::Components;

pub trait ComponentEvent: Sync + Send  {
    fn event(&mut self, ctx: &Context, event: &Event);
}

struct ComponentEventDispatcher {
    components: Components
}

impl ComponentEventDispatcher {
    fn new(components: Components) -> Self { 
        Self { components } 
    }
}



#[async_trait]
impl RawEventHandler for ComponentEventDispatcher {
    async fn raw_event(&self, ctx: Context, ev: Event) {
        for comp in &self.components {
            comp.lock().await.event(&ctx, &ev)
        }
    }
}