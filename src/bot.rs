//! Core de l'application. 
//! L'initialisation du bot et la gestion des composants se fait dans ce module.
use std::sync::Arc;

use futures_locks::RwLock;
use serenity::{Client, model::id::{ApplicationId, UserId}, prelude::GatewayIntents, prelude::RawEventHandler};
use crate::{component_system::{self as cmp, ComponentExt, manager::{Manager, ArcManager}, components::test_component2}, config::Config};
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
    _components: ArcManager,
    _new_components: RwLock<Vec<Arc<dyn new_cmp::Component>>>
}

impl Bot {
    /// Crée un nouveau bot et l'initialise.
    pub async fn new(config: &Config) -> Result<Bot> {
        let manager = RwLock::new(Manager::new());
        let owners_id = config.owners
            .iter()
            .map(|id| id.parse::<u64>().unwrap())
            .map(|id| UserId(id))
            .collect::<Vec<_>>();
        let app_id = ApplicationId(config.app_id);
        let perms = config.permissions;
        // {
        //     use cmp::components::*;
        //     let mut manager_instance = manager.write().await;
        //     let moderation = Moderation::new(app_id).to_arc();
        //     // AJOUTER LES COMPOSANTS ICI A LA SUITE
        //     manager_instance
        //         .add_component(Misc::new(app_id, config.permissions, manager.clone()).to_arc())
        //         .add_component(Tickets::new().to_arc())
        //         .add_component(Help::new(manager.clone()).to_arc())
        //         .add_component(moderation.clone())
        //         .add_component(Autobahn::new(moderation).to_arc())
        //         .add_component(SlashCommands::new(manager.clone(), owners_id, app_id).to_arc());
        // };
        let ref_container = RwLock::new(new_cmp::ComponentContainer::new());
        {
            let mut container = ref_container.write().await;
            // container.add_component(test_component2::Test);
            container.add_component(cmp::components::Help::new(ref_container.clone()));
            container.add_component(cmp::components::Moderation::new());
            container.add_component(cmp::components::Tickets::new());
            container.add_component(cmp::components::SlashCommand::new(app_id, ref_container.clone(), owners_id));
            container.add_component(cmp::components::Misc::new(app_id, perms, ref_container.clone()));
        }
        // let new_components: RwLock<Vec<Arc<dyn new_cmp::Component>>> = RwLock::new(vec![
        //     Arc::new(cmp::components::test_component2::Test),
        //     Arc::new(cmp::components::SlashCommand::new(container.clone(), owners_id)),
        // ]);
        
        let event_container = cmp::EventDispatcher::new(manager.clone());
        let client = Client::builder(&config.token, GatewayIntents::non_privileged())
            .raw_event_handler(ref_container.read().await.get_event_dispatcher())
            .application_id(config.app_id)
            .await?;
        Ok(Bot{
            client,
            _components: manager,
            _new_components: RwLock::new(vec![])
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