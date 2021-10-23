
use std::sync::Arc;

use serenity::{Client, client::bridge::gateway::GatewayIntents};
use crate::{config::Config, middleware as mw};

type Result<T> = serenity::Result<T>;

struct MiddlewareHandler {
    pub middlewares: Vec<mw::ArcMiddleware>,
    pub framework: mw::Framework, 
    pub event_container: mw::EventContainer
}
impl MiddlewareHandler {
    pub fn new(framework: mw::Framework) -> Self {
        MiddlewareHandler {
            middlewares: Vec::new(),
            framework,
            event_container: mw::EventContainer::init(),
        }
    }
    pub fn add_middleware(mut self, mw_arc: mw::ArcMiddleware) -> Self {
        self.framework.add_middleware(Arc::clone(&mw_arc));
        self.event_container.add_middleware(Arc::clone(&mw_arc));
        self.middlewares.push(Arc::clone(&mw_arc));
        self
    }
    // fn add_command_group(&mut self)
}

pub struct Bot {
    client: Client,
    _middlewares: Vec<mw::ArcMiddleware>
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = mw::Framework::new('~');
        let mwh = MiddlewareHandler::new(framework)
            .add_middleware(mw::to_arc(mw::BotStart::new()));
            
        let MiddlewareHandler{middlewares,framework,event_container} = mwh;

        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .raw_event_handler(event_container)
            .await?;
        Ok(Bot{
            client,
            _middlewares: middlewares
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}