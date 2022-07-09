//! Core de l'application. 
//! L'initialisation du bot et la gestion des composants se fait dans ce module.

use futures_locks::RwLock;
use serenity::{Client, model::id::{ApplicationId, UserId}, prelude::GatewayIntents};
use crate::{components as cmp, config::Config};
use opencdd_components as new_cmp;

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
    cmp_container: RwLock<new_cmp::ComponentContainer>
}

impl Bot {
    /// Crée un nouveau bot et l'initialise.
    pub async fn new(config: &Config) -> Result<Bot> {
        let owners_id = config.owners
            .iter()
            .map(|id| id.parse::<u64>().unwrap())
            .map(|id| UserId(id))
            .collect::<Vec<_>>();
        let app_id = ApplicationId(config.app_id);
        let perms = config.permissions;
        let ref_container = RwLock::new(new_cmp::ComponentContainer::new());
        {
            let mut container = ref_container.write().await;
            container.add_component(cmp::Help::new(ref_container.clone()));
            let modo = container.add_component(cmp::Moderation::new());
            container.add_component(cmp::Tickets::new());
            container.add_component(cmp::SlashCommand::new(app_id, ref_container.clone(), owners_id));
            container.add_component(cmp::Misc::new(app_id, perms, ref_container.clone()));
            container.add_component(cmp::Autobahn::new(modo));
        }
        let client = Client::builder(&config.token, GatewayIntents::non_privileged())
            .raw_event_handler(ref_container.read().await.get_event_dispatcher())
            .application_id(config.app_id)
            .await?;
        Ok(Bot{
            client,
            cmp_container: ref_container
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