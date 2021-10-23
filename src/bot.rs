
use std::sync::Arc;

use serenity::{Client, client::bridge::gateway::GatewayIntents};
use crate::{config::Config, component as mw};

type Result<T> = serenity::Result<T>;

struct ComponentHandler {
    pub components: Vec<mw::ArcComponent>,
    pub framework: mw::Framework, 
    pub event_container: mw::EventContainer
}
impl ComponentHandler {
    pub fn new(framework: mw::Framework) -> Self {
        ComponentHandler {
            components: Vec::new(),
            framework,
            event_container: mw::EventContainer::init(),
        }
    }
    pub fn add_component(mut self, mw_arc: mw::ArcComponent) -> Self {
        self.framework.add_component(Arc::clone(&mw_arc));
        self.event_container.add_component(Arc::clone(&mw_arc));
        self.components.push(Arc::clone(&mw_arc));
        self
    }
    // fn add_command_group(&mut self)
}

pub struct Bot {
    client: Client,
    _components: Vec<mw::ArcComponent>
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = mw::Framework::new('~');
        let mwh = ComponentHandler::new(framework)
            .add_component(mw::to_arc(mw::BotStart::new()));
            
        let ComponentHandler{components,framework,event_container} = mwh;

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