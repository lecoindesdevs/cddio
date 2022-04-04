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
use crate::component_system::command_parser as cmd;
use crate::component_system::manager::ArcManager;
use super::{utils};


pub struct Misc {
    node: cmd::Node,
    app_id: ApplicationId,
    bot_permissions: u64,
    cmp_manager: ArcManager
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
                println!("{} is connected!", ready.user.name);
                let perms = Permissions::from_bits(self.bot_permissions)
                    .map(|v| {
                        println!("Permission(s) demandé par le bot: {}", v);
                        v
                    })
                    .unwrap_or_else(|| {
                        println!("Permission du bot dans la configuration invalide. Utilisation de la permission par défaut.");
                        Permissions::empty()
                    });
                let invite = ready.user.invite_url(&ctx.http, perms).await;
                
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
    pub fn new(app_id: ApplicationId, bot_permissions: u64, cmp_manager: ArcManager) -> Misc {
        Misc {
            node: cmd::Node::new()
                .add_command(cmd::Command::new("ping")
                    .set_help("Permet d'avoir une réponse du bot")
                )
                .add_command(cmd::Command::new("list_components")
                    .set_help("Liste les composants du bot et leurs commandes") 
                ),
            app_id,
            bot_permissions,
            cmp_manager
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
        if !self.node.has_command_name(app_cmd.fullname_vec().into_iter()) {
            return Ok(());
        }
        if let None = app_cmd.get_guild_id() {
            return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        

        let command_name = app_cmd.fullname();
        let msg = match command_name.as_str() {
            "ping" => message::success("Pong!"),
            "list_components" => self.list_components().await,
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
    async fn list_components(&self) -> message::Message {
        let mut msg = message::custom_embed("Liste des composants", "", 0x1ed760);
        let embed = msg.last_embed_mut().unwrap();
        let data = self.cmp_manager.read().await;
        for c in data.get_components().iter() {
            let mut cmd_list = String::new();
            match c.node() {
                Some(node) => {
                    for cmd in node.list_commands_names() {
                        cmd_list.push_str(&format!("`{}`\n", cmd));
                    }
                },
                None => {
                    cmd_list.push_str("Aucune commande disponible.");
                }
            }
            embed.field(format!("Composant {}", c.name()), cmd_list, true);
        }
        msg
        // message::success("Pong!")
    }
}