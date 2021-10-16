use std::sync::Arc;

use serenity::{Client, async_trait, client::{ClientBuilder, Context, EventHandler, bridge::gateway::GatewayIntents}, framework, model::gateway::Ready, model::{Permissions, prelude::Gateway}};
use crate::{commands, config::Config};

type Result<T> = serenity::Result<T>;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let http = Arc::clone(&ctx.http);
        match ready.user.invite_url(http, Permissions::empty()).await {
            Ok(v) => println!("Invitation: {}", v),
            Err(e) => println!("Unable to create invitation link: {}", e.to_string()),
        }
    }
}

pub struct Bot {
    client: Client
}

impl Bot {
    pub async fn new(config: &Config) -> Result<Bot> {
        let framework = framework::StandardFramework::new()
            .configure(|c| c
                .prefix(&config.prefix)
            );
        let framework = commands::set_commands(framework);

        let client = Client::builder(&config.token)
            .framework(framework)
            .intents(GatewayIntents::all())
            .event_handler(Handler)
            .await?;
        Ok(Bot{
            client
        })
    }
    pub async fn start(&mut self) -> Result<()> {
        self.client.start().await
    }
}