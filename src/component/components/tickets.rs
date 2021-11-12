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
    prefix: String,
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
        self.r_event(ctx, evt).await
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
                        .add_param(cmd::Argument::new("prefix")
                            .set_required(true)
                            .set_help("Prefix du salon du ticket (ex: ticket)")
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
    async fn r_event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
        use serenity::model::event::Event::*;
        use serenity::model::prelude::*;

        match evt {
            InteractionCreate(evt) => {
                let msg_cmp = match evt.interaction.clone().message_component() {
                    Some(v) => v,
                    None => return Ok(())
                };
                if msg_cmp.data.custom_id != "menu_type_crea_ticket" {
                    return Ok(())
                }
                let value = &msg_cmp.data.values[0];
                let data = self.data.read().await;
                let data = data.read();
                let cat = match data.categories.iter().find(|cat| cat.name == *value) {
                    Some(v) => v,
                    None => return Ok(())
                };
                let member = match &msg_cmp.member {
                    Some(v) => v,
                    None => return Ok(())
                };
                let username  = member.display_name().to_string();
                let guild_id = match &msg_cmp.guild_id {
                    Some(guild) => guild,
                    None => return Ok(())
                };
                let roles = guild_id.roles(ctx).await.unwrap();
                let modo = match roles.iter().find(|role| role.1.name == "Modérateur") {
                    Some(v) => v.0.clone(),
                    None => return Ok(())
                };
                let everyone = RoleId(guild_id.0);
                let new_channel = match guild_id.create_channel(ctx, |ch| {
                    let permissions = vec![
                    // Personne ne peut voir le channel...
                    PermissionOverwrite {
                        allow: Default::default(),
                        deny: Permissions::READ_MESSAGES,
                        kind: PermissionOverwriteType::Role(everyone),
                    },
                    // ...excepté les modérateurs et au dessus...
                    PermissionOverwrite {
                        allow: Permissions::READ_MESSAGES,
                        deny: Default::default(),
                        kind: PermissionOverwriteType::Role(modo),
                    },
                    // ...et le creéateur du ticket
                    PermissionOverwrite {
                        allow: Permissions::READ_MESSAGES,
                        deny: Default::default(),
                        kind: PermissionOverwriteType::Member(member.user.id),
                    }];
                    ch
                        .name(format!("{}_{}", cat.prefix, username))
                        .category(cat.id)
                        .permissions(permissions);
                    
                    ch
                }).await {
                    Ok(v) => Some(v),
                    Err(e) => {
                        println!("Erreur lors de la création du channel: {:?}", e);
                        None
                    }
                };
                match msg_cmp.create_interaction_response(ctx, |resp| 
                    resp
                        .interaction_response_data(|resp_data|
                            resp_data
                                .content(if let Some(v) = new_channel {
                                    format!("Le ticket a bien été créé.\n\nVous pouvez le rejoindre en cliquant sur le lien suivant: {}", v.mention())
                                } else {
                                    "Erreur lors de la création du channel".to_string()
                                })
                                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        )
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                ).await {
                    Ok(_) => (),
                    Err(e) => println!("Erreur lors de la création de la réponse: {:?}", e)
                };
                match self.update_message_components(ctx).await {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error updating message components: {}", e)
                }
                
            }
            _ => {}
        } 
        Ok(())
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
                    match self.update_message_components(ctx).await {
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
    fn create_components(&self, cmps: &mut serenity::builder::CreateComponents, categories: &Vec<CategoryTicket>) {
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
        cmps.add_action_row(act);
    }
    async fn update_message_components(&self, ctx: &Context) -> serenity::Result<()> {
        
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

            msg.components(|cmps| {
                self.create_components(cmps, categories);
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
                let prefix = matched.get_parameter("prefix").unwrap().value.to_string();
                let desc = match matched.get_parameter("desc") {
                    Some(desc) => Some(desc.value.to_string()),
                    None => None
                };
                if let Err(e) = self.add_category(ctx, msg, name, desc, id, prefix).await {
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
    async fn add_category(&self, ctx: &Context, msg: &Message, name: String, desc: Option<String>, id: u64, prefix: String) -> serenity::Result<()> {
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
                prefix,
                emoji: None,
            });
        }
        err_println!(self.update_message_components(ctx).await, "tickets: unable to update message after adding a category.\n{:?}");
        common::send_success_message(ctx, msg, format!("La catégorie {} a été ajoutée.", name)).await
    }
    async fn remove_category(&self, ctx: &Context, msg: &Message, name: String) -> serenity::Result<()> {
        let i = match self.data.read().await.read().categories.iter().position(|v| v.name == name) {
            Some(i) => i,
            None => return common::send_error_message(ctx, msg, format!("La catégorie {} n'existe pas.", name)).await
        };
        self.data.write().await.write().categories.swap_remove(i);
        err_println!(self.update_message_components(ctx).await, "tickets: unable to update message after deleting a category.\n{:?}");
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