//! Le composant misc est une sorte de composant fourre tout, lors qu'une commande ou un événement seul ne necéssite pas de composant a lui tout seul. 
//! Pensez à regarder le domaine des autres composants voir si ce que vous voulez implémenter ne correspondrait pas à un des composants déjà existant.
//! **Attention toutefois** : le composant misc ne doit rien enregistrer et ne doit pas posséder de configuration. 
//! Une action (commande ou événement) dans ce composant doit se suffire à elle-même.

use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use super::super::{CommandMatch, Component, FrameworkConfig};
use crate::component::command_parser::{self as cmd, ParseError};
use super::{utils};


pub struct Misc {
    group_match: cmd::Group
}

#[derive(Serialize, Deserialize, Debug)]
struct DataTest{
    don1: String,
    don2: i32
}

#[async_trait]
impl Component for Misc {
    fn name(&self) -> &'static str {
        "misc"
    }
    async fn command(&self, _: &FrameworkConfig, ctx: &Context, msg: &Message) -> CommandMatch {
        let args = cmd::split_shell(&msg.content[1..]);
        let matched = match utils::try_match(ctx, msg, &self.group_match, args).await {
            Ok(v) => v,
            Err(e) => return e
        };
        match matched.get_command() {
            "ping" => {
                match utils::has_permission(ctx, msg, matched.permission).await {
                    Ok(true) => Self::send_message(ctx, msg, "Pong!").await,
                    Ok(false) => {
                        match utils::send::no_perm(ctx, msg.channel_id).await {
                            Ok(_) => CommandMatch::Matched,
                            Err(e) => return CommandMatch::Error(e.to_string())
                        }
                    },
                    Err(e) => e
                }
            },
            "data" => {
                let mut data = utils::Data::new("misc", DataTest{ don1: "yes".to_string(), don2: 32 });
                let mut guard = data.write();
                guard.don1 = "no".to_string();
                CommandMatch::Matched
            }
            _ => unreachable!()
        }
    }

    async fn event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
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
                .add_command(cmd::Command::new("data")
                    .set_help("Teste l'enregistrement des donénes")
                )
        }
    }
    pub async fn send_message(ctx: &Context, msg: &Message, txt: &str) -> CommandMatch{
        match msg.channel_id.say(&ctx.http, txt).await {
            Ok(_) => CommandMatch::Matched,
            Err(e) => CommandMatch::Error(e.to_string()),
        }
    }
}