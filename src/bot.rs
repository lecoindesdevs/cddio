
use std::sync::Arc;

use serenity::{Client, client::bridge::gateway::GatewayIntents};
use crate::{config::Config, component as cmp};

type Result<T> = serenity::Result<T>;

struct ComponentHandler {
    pub components: Vec<cmp::ArcComponent>,
    pub framework: cmp::Framework, 
    pub event_container: cmp::EventContainer
}
impl ComponentHandler {
    pub fn new(framework: cmp::Framework) -> Self {
        ComponentHandler {
            components: Vec::new(),
            framework,
            event_container: cmp::EventContainer::init(),
        }
    }
    pub fn add_component(mut self, cmp_arc: cmp::ArcComponent) -> Self {
        self.framework.add_component(Arc::clone(&cmp_arc));
        self.event_container.add_component(Arc::clone(&cmp_arc));
        self.components.push(Arc::clone(&cmp_arc));
        self
    }
    pub fn add_help(self) -> Self {
        let help = cmp::to_arc_mut(cmp::components::Help::new(self.components.clone()));
        self.add_component(help)
    }
    // fn add_command_group(&mut self)
}

pub struct Bot {
    client: Client,
    _components: Vec<cmp::ArcComponent>
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = cmp::Framework::new(config.prefix);
        let cmph = ComponentHandler::new(framework)
        // AJOUTER LES COMPOSANTS ICI A LA SUITE
            .add_component(cmp::to_arc_mut(cmp::components::Misc::new()))
            .add_help();
            
        let ComponentHandler{components,framework,event_container} = cmph;

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
    
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}