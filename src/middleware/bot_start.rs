use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use super::{CommandMatch, Middleware};



pub struct BotStart;

#[async_trait]
impl Middleware for BotStart {
    fn name(&self) -> &'static str {
        "Bot Start"
    }

    async fn command(&mut self, ctx: &Context, msg: &Message) -> CommandMatch {
        if msg.content == "~ping" {
            match msg.channel_id.say(&ctx.http, "pong!").await {
                Ok(_) => CommandMatch::Matched,
                Err(e) => CommandMatch::Error(e.to_string()),
            }
        } else {
            CommandMatch::NotMatched
        }
    }

    async fn event(&mut self, ctx: &Context, evt: &Event) -> Result<(), String> {
        if let Event::Ready(ReadyEvent{ready, ..}) = evt {
            let (username, invite) = { 
                (ready.user.name.clone(), ready.user.invite_url(&ctx.http, Permissions::empty()).await)
            };
            println!("{} is connected!", username);
            match invite {
                Ok(v) => println!("Invitation: {}", v),
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    }
}

impl BotStart {
    pub fn new () -> BotStart {
        BotStart
    }
}
