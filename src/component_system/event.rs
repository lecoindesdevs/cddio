
use std::ops::Deref;
use serenity::async_trait;
use serenity::client::{Context, RawEventHandler};
pub use serenity::model::event::Event;

use super::manager::ArcManager;

/// Event handler qui dispatch les events aux composants.
/// 
/// Dès qu'un event est reçu par le client, il est envoyé à tous les composants enregistrés.
/// C'est au composant de traiter quel type d'event il a besoin.
pub struct EventDispatcher {
    cmp_manager: ArcManager,
}

impl EventDispatcher {
    pub fn new(cmp_manager: ArcManager) -> EventDispatcher {
        EventDispatcher{
            cmp_manager,
        }
    }
}

#[async_trait]
impl RawEventHandler for EventDispatcher {
    async fn raw_event(&self, ctx: Context, evt: Event) {
        let components = self.cmp_manager.read().await.get_components().clone();
        tokio::spawn(async move {
            for component in components {
                let component = component.deref();
                if let Err(what) = component.event(&ctx, &evt).await {
                    println!("[{}] Module {} command error: {}\nEvent: {:?}\n\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
                        component.name(),
                        what,
                        evt
                    );
                }
            }
        });
    }
}
