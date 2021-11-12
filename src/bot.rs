//! Core de l'application. 
//! L'initialisation du bot et la gestion des composants se fait dans ce module.
use serenity::{Client, client::bridge::gateway::GatewayIntents};
use crate::{component::{self as cmp, manager::{Manager, ArcManager}}, config::Config, util::ArcRw};
use cmp::Component;
type Result<T> = serenity::Result<T>;

/// Structure du bot.
/// 
/// Il s'agit de la classe mère De l'application. 
/// 
/// Se charge d'initialiser le client serenity en incorporant un command handler ([`crate::component::Framework`]) et un event handler ([`crate::component::EventDispatcher`]) puis de lancer le bot.
/// 
/// Le bot est composé de plusieurs composants qui sont chargés dans le framework et dans l'event container par le biais de la classe [`ComponentHandler`].
pub struct Bot {
    /// Client discord de serenity
    client: Client,
    /// Handler des composants.
    /// Actuellement un vecteur mais prochainement un gestionnaire est prévu.
    _components: ArcManager
}

impl Bot {
    /// Crée un nouveau bot et l'initialise.
    pub async fn new(config: &Config) -> Result<Bot> {
        let manager = ArcRw::new(Manager::new());
        {
            use cmp::components::*;
            let mut manager_instance = manager.write().await;
            // AJOUTER LES COMPOSANTS ICI A LA SUITE
            manager_instance.add_component(Misc::new().to_arc());
            manager_instance.add_component(Tickets::new().to_arc());
            manager_instance.add_component(Help::new(manager.clone()).to_arc());
        };
        
        let framework = cmp::Framework::new(config.prefix, manager.clone());
        let event_container = cmp::EventDispatcher::new(manager.clone());
        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .raw_event_handler(event_container)
            .application_id(config.app_id)
            .await?;
        Ok(Bot{
            client,
            _components: manager
        })
    }
    /// Lance le bot.
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}

//TODO: Enregistrer la configuration du bot lors du drop de ce dernier
impl Drop for Bot {
    fn drop(&mut self) {
        println!("Bot dropped");
    }
}