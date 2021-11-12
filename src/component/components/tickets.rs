use std::path::PathBuf;

use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, http};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::event::{Event, ReadyEvent};
use crate::component::{self as cmp, FrameworkConfig, command_parser as cmd};
use super::common;

use super::common::Data;

#[derive(Serialize, Deserialize, Default, Debug)]
struct DataTickets {
    tickets: Vec<String>,
    msg_react: Option<(u64, u64)>,
}

pub struct Tickets {
    data: RwLock<Data<DataTickets>>,
    group_match: cmd::Group,
    archives_folder: PathBuf
}
#[async_trait]
impl crate::component::Component for Tickets {
    fn name(&self) -> &str {
        "tickets"
    }
    async fn command(&self, fw_config: &FrameworkConfig, ctx: &Context, msg: &Message) -> cmp::CommandMatch {
        self.r_command(fw_config, ctx, msg).await
    }
    async fn event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
        Ok(())
    }
    fn group_parser(&self) -> Option<&cmd::Group> {
        Some(&self.group_match)
    }
}

impl Tickets {
    pub fn new() -> Self {
        Tickets{
            group_match: cmd::Group::new("tickets")
                .set_help("Gestion des tickets")
                .set_permission("owners")
                .add_group(cmd::Group::new("set")
                    .set_help("Changer les paramètres")
                    .add_command(cmd::Command::new("channel_msg")
                        .set_help("Change le salon ou poser le message de création de ticket")
                        .add_param(cmd::Argument::new("id")
                            .set_required(true)
                            .set_help("Identifiant du message")
                        ))
                    .add_command(cmd::Command::new("category")
                        .set_help("Change la catégorie où les tickets vont se placer.")
                        .add_param(cmd::Argument::new("id")
                            .set_required(true)
                            .set_help("Identifiant de la catégorie")
                    ))
                )
                .add_command(cmd::Command::new("list")
                    .set_help("Liste les tickets")
                ),
            data: match Data::from_file_default("tickets") {
                Ok(data) => RwLock::new(data),
                Err(e) => panic!("Data tickets: {:?}", e)
            },
            archives_folder: common::DATA_DIR.join("archives"),
        }
    }
    async fn r_command(&self, fw_config: &FrameworkConfig, ctx: &Context, msg: &Message) -> cmp::CommandMatch {
        let args = cmd::split_shell(&msg.content[1..]);
        let matched = match common::try_match(ctx, msg, &self.group_match, args).await {
            Ok(v) => v,
            Err(e) => return e
        };
        match (matched.get_groups(), matched.get_command()) {
            (["tickets", "set"], "channel_msg") => return self.set_channel(ctx, msg, &matched).await,
            (["tickets", "set"], "category") => todo!(),
            (["tickets"], "list") => todo!(),
            _ => unreachable!()
        };
        cmp::CommandMatch::Matched
    }
    async fn delete_old_creation_message(&self, ctx: &Context, msg: &Message) -> Result<(), serenity::Error> {
        let old_msg = self.data.read().await.read().msg_react;
        if let Some((channel_id, msg_id)) = old_msg {
            ctx.http.delete_message(channel_id, msg_id).await
        } else {
            Ok(())
        }
    }
    async fn set_channel(&self, ctx: &Context, msg: &Message, matched: &cmd::matching::Command<'_>) -> cmp::CommandMatch {
        if let Err(e) =  self.delete_old_creation_message(ctx, msg).await {
            eprintln!("tickets: unable to delete previous message.\n{:?}", e)
        }
        let id: u64 = match matched.get_parameter("id").unwrap().value.parse() {
            Ok(v) => v,
            Err(e) => return cmp::CommandMatch::Error(format!("{:?}", e))
        };
        let mut data = self.data.write().await;
        let mut data = data.write();
        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => {
                common::send_error_message(ctx, msg, "Vous devez être dans un serveur pour utiliser cette commande.").await;
                return cmp::CommandMatch::Matched;
            },
        };
        let channel = guild_id.channels(ctx).await.unwrap().into_iter().find(|channel| channel.0.0 == id);
        if let Some((channel, _)) = channel {
            let msg_tickets= channel.say(ctx, "huehuehu").await.unwrap();
            
            let channel_name = match channel.name(ctx).await {
                Some(name) => name,
                None => "que vous avez renseigné".to_string()
            };
            match msg_tickets.react(ctx, '✅').await {
                Ok(_) => {
                    common::send_success_message(ctx, msg, format!("Le message de création de ticket a été mis à jour dans le salon {}.", channel_name)).await;
                    data.msg_react = Some((channel.0, msg_tickets.id.0));
                },
                Err(e) => {
                    common::send_error_message(ctx, msg, format!("{}", e.to_string())).await;
                }
            }
        } else {
            common::send_error_message(ctx, msg, "Le salon n'existe pas.").await;
        }
        cmp::CommandMatch::Matched
    }
}