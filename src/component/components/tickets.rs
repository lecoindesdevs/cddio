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
    msg_react: u64,
    tickets_catergory: String,
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
                Ok(data) => data,
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
        cmp::CommandMatch::Matched
    }
}