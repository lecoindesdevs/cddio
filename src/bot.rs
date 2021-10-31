//! Core de l'application. 
//! L'initialisation du bot et la gestion des composants se fait dans ce module.
use std::sync::Arc;

use serenity::{Client, client::bridge::gateway::GatewayIntents};
use crate::{config::Config, component as cmp};

type Result<T> = serenity::Result<T>;

/// Helper pour l'initialisation du bot.
/// 
/// Le but est de faciliter l'intégration des composants dans le framework et l'event container.
struct ComponentHandler<'a> {
    pub components: Vec<cmp::ArcComponent>,
    pub framework: cmp::Framework, 
    pub event_container: cmp::EventDispatcher,
    pub config: &'a Config,
}
impl<'a> ComponentHandler<'a> {
    /// Créer une instrance de ComponentHandler.
    /// Seules le framework et le config sont requis. Le reste est généré automatiquement.
    pub fn new(framework: cmp::Framework, config:&'a Config) -> Self {
        ComponentHandler {
            components: Vec::new(),
            framework,
            event_container: cmp::EventDispatcher::new(),
            config
        }
    }
    /// Ajoute un composant.
    /// 
    /// La fonction a pour but de simplifier l'intégration des composants dans le framework et l'event container.
    pub fn add_component(mut self, cmp_arc: cmp::ArcComponent) -> Self {
        self.framework.add_component(Arc::clone(&cmp_arc));
        self.event_container.add_component(Arc::clone(&cmp_arc));
        self.components.push(Arc::clone(&cmp_arc));
        self
    }
    /// Ajoute le composant help.
    /// 
    /// S'agissant d'un composant spécial, la fonction donne les accès des composants après leur ajout au composant help.
    /// Les composants doivent être ajoutés __avant__ le composant help pour qu'ils soit pris en compte.
    pub fn add_help(self) -> Self {
        let help = cmp::to_arc_mut(cmp::components::Help::new(self.components.clone()));
        self.add_component(help)
    }
}
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
    _components: Vec<cmp::ArcComponent>
}

impl Bot {
    /// Crée un nouveau bot et l'initialise.
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = cmp::Framework::new(config.prefix);
        let cmph = ComponentHandler::new(framework, &config)
        // AJOUTER LES COMPOSANTS ICI A LA SUITE
            .add_component(cmp::to_arc_mut(cmp::components::Misc::new()))
        // LES COMPOSANTS AJOUTES APRES CETTE LIGNE NE SERONT PAS PRIS EN COMPTE PAR LE COMPOSANT HELP
            .add_help();
            
        let ComponentHandler{components,framework,event_container, config: _} = cmph;

        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .raw_event_handler(event_container)
            .await?;
        Ok(Bot{
            client,
            _components: components
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