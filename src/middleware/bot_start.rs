use futures::lock::Mutex;
use serenity::async_trait;
use serenity::client::Context;
use serenity::framework::standard::{CommandGroup, CommandResult};
use serenity::model::channel::Message;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use serenity::framework::standard::macros::command;
use super::{SubEvent, Middleware};

pub struct BotStart {
    command_group: CommandGroup
}

#[async_trait]
impl SubEvent for BotStart {
    async fn raw_event(&mut self, ctx: &Mutex<Context>, evt: &Mutex<Event>) {
        let evt = evt.lock().await.clone();
        if let Event::Ready(ReadyEvent{ready, ..}) = evt {
            let (username, invite) = { 
                let ctx = ctx.lock().await;
                (ready.user.name.clone(), ready.user.invite_url(&ctx.http, Permissions::empty()).await)
            };
            println!("{} is connected!", username);
            match invite {
                Ok(v) => println!("Invitation: {}", v),
                Err(e) => println!("Unable to create invitation link: {}", e.to_string()),
            }
        }
    }
}

impl Middleware for BotStart {
    fn name(&self) -> &'static str {
        "Bot Start"
    }

    fn command_group<'a>(&'a self) -> Option<&'a CommandGroup> {
        Some(&self.command_group)
    }
}
impl BotStart {
    pub fn new() -> BotStart {
        use serenity::framework::standard::*;
        BotStart {
            command_group: CommandGroup{
                name: "test",
                options: &GroupOptions{
                    commands: &[
                        &Command{
                            fun: ping,
                            options: &CommandOptions{
                                ..Default::default()
                            },
                        }
                    ],
                    .. Default::default()
                },
            }
        }
    }
}


#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("ping command");
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}
