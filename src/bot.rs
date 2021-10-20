use crate::event::EventListenerContainer;
use serenity::{Client, client::bridge::gateway::GatewayIntents, framework::StandardFramework};
use crate::{commands, config::Config};

type Result<T> = serenity::Result<T>;


pub struct Bot {
    client: Client
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = StandardFramework::new()
            .configure(|c| c
                .prefix(&config.prefix)
            );
        let framework = commands::set_commands(framework);
        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .raw_event_handler(EventListenerContainer::init())
            .await?;
        Ok(Bot{
            client
        })
    }
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}