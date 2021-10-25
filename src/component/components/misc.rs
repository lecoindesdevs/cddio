use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use super::super::{CommandMatch, Component, FrameworkConfig};
use crate::component::command_parser::{self as cmd, ParseError};


pub struct Misc {
    group_match: cmd::Group
}

#[async_trait]
impl Component for Misc {
    fn name(&self) -> &'static str {
        "misc"
    }

    async fn command(&mut self, _: &FrameworkConfig, ctx: &Context, msg: &Message) -> CommandMatch {
        let args = cmd::split_shell(&msg.content[1..]);
        let matched = match self.group_match.try_match(&args) {
            Ok(v) => v,
            Err(ParseError::NotMatched) => return CommandMatch::NotMatched,
            Err(e) => return CommandMatch::Error(e.to_string())
        };
        match matched.get_command() {
            "ping" => Self::send_text(ctx, msg, "pong!").await,
            _ => unreachable!()
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
    fn group_parser(&self) -> Option<&cmd::Group> {
        Some(&self.group_match)
    }
}

impl Misc {
    pub fn new () -> Misc {
        Misc{
            group_match: cmd::Group::new("misc")
                .set_help("Commande diverse, sans catégorie, ou de test")
                .add_command(cmd::Command::new("ping")
                    .set_help("Permet d'avoir une réponse du bot")
                )
        }
    }
    pub async fn send_text(ctx: &Context, msg: &Message, txt: &str) -> CommandMatch{
        match msg.channel_id.say(&ctx.http, "pong!").await {
            Ok(_) => CommandMatch::Matched,
            Err(e) => CommandMatch::Error(e.to_string()),
        }
    }
}