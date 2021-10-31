
use serenity::async_trait;
use serenity::client::{Context, RawEventHandler};
pub use serenity::model::event::Event;

use super::ArcComponent;

/// Event handler qui dispatch les events aux composants.
/// 
/// Dès qu'un event est reçu par le client, il est envoyé à tous les composants enregistrés.
/// C'est au composant de traiter quel type d'event il a besoin.
#[derive(Default)]
pub struct EventDispatcher {
    event_listeners: Vec<ArcComponent>,
}

impl EventDispatcher {
    pub fn new() -> EventDispatcher {
        EventDispatcher{
            event_listeners: Vec::new(),
        }
    }
    /// Enregistre un composant à la liste des composants.
    pub fn add_component(&mut self, event_listener: ArcComponent) {
        self.event_listeners.push(event_listener);
    }
}

#[async_trait]
impl RawEventHandler for EventDispatcher {
    async fn raw_event(&self, ctx: Context, evt: Event) {
        for component in &self.event_listeners {
            let mut component = component.lock().await;
            if let Err(what) = component.event(&ctx, &evt).await {
                println!("[{}] Module {} command error: {}\nEvent: {:?}\n\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
                    component.name(),
                    what,
                    evt
                );
            }
        }
    }
}
