
use std::sync::Arc;

use futures::lock::Mutex;
use serenity::{Client, client::bridge::gateway::GatewayIntents, framework::{StandardFramework, standard::macros::group}};
use crate::{config::Config, middleware as mw};

type Result<T> = serenity::Result<T>;



struct MiddlewareHandler {
    pub middlewares: Vec<mw::ArcMiddleware>,
    pub framework: StandardFramework, 
    pub event_container: mw::EventContainer
}
impl MiddlewareHandler {
    pub fn add_middleware(&mut self, middleware: impl 'static+mw::Middleware) {
        if let Some(v) = middleware.command_group() {
            self.framework = self.framework.group(v);
        }
        let mw_arc = Arc::new(Mutex::new(middleware));
        self.event_container.add_event_listener(Arc::clone(&(mw_arc as mw::ArcEvent)));
        self.middlewares.push(mw_arc);
    }
}

pub struct Bot {
    client: Client,
    middlewares: Vec<mw::ArcMiddleware>
}
#[group]
struct GTest;

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = StandardFramework::new()
            .configure(|c| c
                .prefix(&config.prefix)
            );
        let mut mwh = MiddlewareHandler {
            middlewares: Vec::new(),
            framework,
            event_container: mw::EventContainer::init(),
        };

        mwh.add_middleware(mw::BotStart::new());

        let MiddlewareHandler{middlewares,framework,event_container} = mwh;
        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .raw_event_handler(event_container)
            .await?;
        Ok(Bot{
            client,
            middlewares
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}