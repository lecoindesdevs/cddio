//! Le composant misc est une sorte de composant fourre tout, lors qu'une commande ou un événement seul ne necéssite pas de composant a lui tout seul. 
//! Pensez à regarder le domaine des autres composants voir si ce que vous voulez implémenter ne correspondrait pas à un des composants déjà existant.
//! **Attention toutefois** : le composant misc ne doit rien enregistrer et ne doit pas posséder de configuration. 
//! Une action (commande ou événement) dans ce composant doit se suffire à elle-même.

use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::event::InteractionCreateEvent;
use serenity::model::id::ApplicationId;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use super::super::{CommandMatch, Component, FrameworkConfig};
use super::utils::app_command::ApplicationCommandEmbed;
use super::utils::message;
use crate::component::command_parser::{self as cmd, ParseError};
use super::{utils};


pub struct Misc {
    node: cmd::Node,
    app_id: ApplicationId,
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
        let matched = match utils::try_match(ctx, msg, &self.node, args).await {
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
            _ => CommandMatch::NotMatched
        }
    }

    async fn event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
        match evt {
            Event::Ready(ReadyEvent { ready, .. }) => {
                let (username, invite) = { 
                    (ready.user.name.clone(), ready.user.invite_url(&ctx.http, Permissions::empty()).await)
                };
                println!("{} is connected!", username);
                match invite {
                    Ok(v) => println!("Invitation: {}", v),
                    Err(e) => return Err(e.to_string()),
                }
                Ok(())
            },
            Event::InteractionCreate(InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await,
            _ => Ok(())
        }
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.node)
    }
}

impl Misc {
    pub fn new(app_id: ApplicationId) -> Misc {
        Misc{
            node: cmd::Node::new()
                .add_command(cmd::Command::new("ping")
                    .set_help("Permet d'avoir une réponse du bot")
                ),
            app_id
        }
    }
    pub async fn send_message(ctx: &Context, msg: &Message, txt: &str) -> CommandMatch{
        match msg.channel_id.say(&ctx.http, txt).await {
            Ok(_) => CommandMatch::Matched,
            Err(e) => CommandMatch::Error(e.to_string()),
        }
    }
    async fn on_applications_command(&self, ctx: &Context, app_command: &ApplicationCommandInteraction) -> Result<(), String> {
        if app_command.application_id != self.app_id {
            // La commande n'est pas destiné à ce bot
            return Ok(());
        }
        let app_cmd = ApplicationCommandEmbed::new(app_command);
        if let None = app_cmd.get_guild_id() {
            return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        let command_name = app_cmd.fullname();
        let msg = match command_name.as_str() {
            "ping" => message::success("Pong!"),
            _ => return Ok(())
        };
        app_command.create_interaction_response(ctx, |resp|{
            *resp = msg.into();
            resp
        }).await.or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e.to_string())
        })
    }
}