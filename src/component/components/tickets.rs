use std::path::PathBuf;

use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::prelude::Mentionable;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::event::Event;
use crate::component::{self as cmp, FrameworkConfig, command_parser as cmd};
use super::common;

use super::common::Data;

macro_rules! err_println {
    (send_error($ctx: ident, $msg: ident, $txt:expr)) => {
        err_println!(common::send_error_message($ctx, $msg, $txt).await, "Error sending error message: {}")
    };
    (send_success($ctx: ident, $msg: ident, $txt:expr)) => {
        err_println!(common::send_success_message($ctx, $msg, $txt).await, "Error sending success message: {}")
    };
    ($result:expr,$msg_format:expr) => {
        {
            match $result {
                Ok(_) => (),
                Err(e) => eprintln!($msg_format, e)
            }
        }
    };
    
    
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct CategoryTicket {
    name: String, 
    id: u64,
    desc: Option<String>,
    emoji: Option<ReactionType>,
    tickets: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct DataTickets {
    msg_react: Option<(u64, u64)>,
    categories: Vec<CategoryTicket>,
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
        match evt {
            
            _ => {}
        } 
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
                .add_group(cmd::Group::new("create_channel")
                    .set_help("Salon de création de tickets")
                    .add_command(cmd::Command::new("set")
                        .set_help("Change le salon")
                        .add_param(cmd::Argument::new("id")
                            .set_required(true)
                            .set_help("Identifiant du message")
                        )
                    )
                )
                .add_group(cmd::Group::new("categories")
                    .set_help("Gestion des catégories de tickets.")
                    .add_command(cmd::Command::new("add")
                        .set_help("Ajoute une catégorie de ticket. À ne pas confondre avec les catégories discord")
                        .add_param(cmd::Argument::new("name")
                            .set_required(true)
                            .set_help("Nom de la catégorie")
                        )
                        .add_param(cmd::Argument::new("id")
                            .set_required(true)
                            .set_help("Identifiant de la catégorie Discord")
                        )
                        .add_param(cmd::Argument::new("desc")
                            .set_required(false)
                            .set_help("Description de la catégorie de ticket")
                        )
                        // .add_param(cmd::Argument::new("emoji")
                        //     .set_required(false)
                        //     .set_help("Emoji décoration")
                        // )
                    )
                    .add_command(cmd::Command::new("remove")
                        .set_help("Supprime une catégorie de ticket")
                        .add_param(cmd::Argument::new("name")
                            .set_required(true)
                            .set_help("Nom de la catégorie")
                        )
                    )
                    .add_command(cmd::Command::new("list")
                        .set_help("Liste les catégories de tickets")
                    )
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
            (["tickets", "create_channel"], "set") => return self.set_channel(ctx, msg, &matched).await,
            (["tickets", "categories"], _) => return self.categories(ctx, msg, &matched).await,
            (["tickets"], "list") => todo!(),
            _ => unreachable!()
        }
    }
    async fn delete_old_creation_message(&self, ctx: &Context, _msg: &Message) -> Result<(), serenity::Error> {
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
        
        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => {
                err_println!(send_error(ctx, msg, "Vous devez être dans un serveur pour utiliser cette commande."));
                return cmp::CommandMatch::Matched;
            },
        };
        let channel = guild_id.channels(ctx).await.unwrap().into_iter().find(|channel| channel.0.0 == id);
        if let Some((channel, _)) = channel {
            let msg_tickets= channel.send_message(ctx, |msg| 
                msg.content("Quel type de ticket souhaitez-vous ouvrir ?")
            ).await;
            let channel_name = match channel.name(ctx).await {
                Some(name) => name,
                None => "que vous avez renseigné".to_string()
            };
            match msg_tickets {
                Ok(msg_sent) => {
                    {
                        let mut data = self.data.write().await;
                        let mut data = data.write();
                        data.msg_react = Some((channel.0, msg_sent.id.0));
                    }
                    match self.update_message_components(ctx, msg).await {
                        Ok(_) => (),
                        Err(e) => eprintln!("tickets: unable to update message.\n{:?}", e)
                    };
                    err_println!(send_success(ctx, msg, format!("Le message de création de ticket a été mis à jour dans le salon {}.", channel_name)));
                },
                Err(e) => err_println!(send_error(ctx, msg, format!("{}", e.to_string())))
            }
        } else {
            err_println!(send_error(ctx, msg, "Le salon n'existe pas."));
        }
        cmp::CommandMatch::Matched
    }
    async fn update_message_components(&self, ctx: &Context, _msg: &Message) -> serenity::Result<()> {
        let data = self.data.read().await;
        let data = data.read();
        let categories = &data.categories;
        let (chan_id, msg_id) = match data.msg_react {
            Some(v) => v,
            None => return Ok(())
        };

        let mut message = match ctx.http.get_message(chan_id, msg_id).await {
            Ok(msg) => msg,
            Err(e) => return Err(e)
        };
        
        message.edit(ctx, |msg| {

            msg.content("Quel type de ticket souhaitez-vous ouvrir ?");
            if categories.is_empty() {
                return msg;
            }
            use serenity::builder::*;

            
            let mut opts = CreateSelectMenuOptions::default();

            for cat in categories {
                let mut opt = CreateSelectMenuOption::default();
                opt.label(&cat.name).value(&cat.name);
                if let Some(desc) = &cat.desc {
                    opt.description(desc);
                }
                if let Some(emoji) = &cat.emoji {
                    opt.emoji(emoji.clone());
                }
                opts.add_option(opt);
            }
            
            let mut menus = CreateSelectMenu::default();
            menus.options(|o| {
                *o = opts;
                o
            });
            menus.custom_id("menu_type_crea_ticket");

            let mut act = CreateActionRow::default();
            act.add_select_menu(menus);

            msg.components(|cmps| {
                cmps.add_action_row(act);
                cmps
            });

            msg
        }).await
    }
    async fn categories(&self, ctx: &Context, msg: &Message, matched: &cmd::matching::Command<'_>) -> cmp::CommandMatch {
        match matched.get_command() {
            "add" => {
                let id: u64 = match matched.get_parameter("id").unwrap().value.parse() {
                    Ok(v) => v,
                    Err(e) => return cmp::CommandMatch::Error(format!("id: paramètre mal formé: {}", e.to_string()))
                };
                let name = matched.get_parameter("name").unwrap().value.to_string();
                let desc = match matched.get_parameter("desc") {
                    Some(desc) => Some(desc.value.to_string()),
                    None => None
                };
                if let Err(e) = self.add_category(ctx, msg, name, desc, id).await {
                    return cmp::CommandMatch::Error(format!("add_category: {:?}", e));
                }
            },
            "remove" => {
                let name = matched.get_parameter("name").unwrap().value.to_string();
                if let Err(e) = self.remove_category(ctx, msg, name).await {
                    return cmp::CommandMatch::Error(format!("add_category: {:?}", e));
                }
            },
            "list" => {
                if let Err(e) = self.list_categories(ctx, msg).await {
                    return cmp::CommandMatch::Error(format!("list_category: {:?}", e));
                }
            },
            _ => unreachable!()
        };
        cmp::CommandMatch::Matched
    }
    async fn add_category(&self, ctx: &Context, msg: &Message, name: String, desc: Option<String>, id: u64) -> serenity::Result<()> {
        if let Some(_) = self.data.read().await.read().categories.iter().find(|v| v.name == name) {
            return common::send_error_message(ctx, msg, format!("La catégorie de ticket {} existe déjà.", name)).await;
        }
        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => return common::send_error_message(ctx, msg, "Vous devez être dans un serveur pour utiliser cette commande.").await
        };
        let (_, guild_channel) = match guild_id.channels(ctx).await.unwrap().into_iter().find(|channel| channel.0.0 == id) {
            Some(v) => v,
            None => return common::send_error_message(ctx, msg, "Le salon n'existe pas.").await
        };
        match guild_channel.kind {
            serenity::model::channel::ChannelType::Category => (),
            _ => return common::send_error_message(ctx, msg, format!("L'id ne pointe pas sur une catégorie mais sur {} de type {:?}.", guild_channel.mention().to_string(), guild_channel.kind)).await
        }
        {
            let mut data = self.data.write().await;
            let mut data = data.write();
            data.categories.push(CategoryTicket{
                name: name.clone(),
                id,
                tickets: Vec::new(),
                desc,
                emoji: None,
            });
        }
        err_println!(self.update_message_components(ctx, msg).await, "tickets: unable to update message after adding a category.\n{:?}");
        common::send_success_message(ctx, msg, format!("La catégorie {} a été ajoutée.", name)).await
    }
    async fn remove_category(&self, ctx: &Context, msg: &Message, name: String) -> serenity::Result<()> {
        let i = match self.data.read().await.read().categories.iter().position(|v| v.name == name) {
            Some(i) => i,
            None => return common::send_error_message(ctx, msg, format!("La catégorie {} n'existe pas.", name)).await
        };
        self.data.write().await.write().categories.swap_remove(i);
        err_println!(self.update_message_components(ctx, msg).await, "tickets: unable to update message after deleting a category.\n{:?}");
        common::send_success_message(ctx, msg, format!("La catégorie {} a été supprimée.", name)).await
    }
    async fn list_categories(&self, ctx: &Context, msg: &Message) -> serenity::Result<()> {
        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => return common::send_error_message(ctx, msg, "Vous devez être dans un serveur pour utiliser cette commande.").await
        };
        let data = self.data.read().await;
        let data = data.read();
        let categories = &data.categories;
        if categories.is_empty() {
            return common::send_error_message(ctx, msg, "Aucune catégorie de ticket n'a été créée.").await;
        }
        let mut cat = Vec::new();
        let channels = match guild_id.channels(ctx).await {
            Ok(channels) => Some(channels),
            Err(_) => None
        };
        
        for category in categories {
            let channel_name = match channels {
                Some(ref v) => match v.iter().find(|channel| channel.0.0 == category.id) {
                    Some(channel) => Some(channel.1.mention().to_string()),
                    None => None
                },
                None => None
            };
            match channel_name {
                Some(name) => cat.push(format!("{} ({})", category.name, name)),
                None => cat.push(format!("{} (id: {})", category.name, category.id)),
            }
        }
        match msg.channel_id.send_message(ctx, |m|
            m.embed(|embed| {
                embed
                    .title("Liste des catégories")
                    .description( cat.join("\n") )
                    .color(0x1ed760)
            })
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }

    }
}