use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use futures::StreamExt;
use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::interactions::Interaction;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::message_component::{ButtonStyle, MessageComponentInteraction};
use serenity::prelude::Mentionable;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::event::Event;

use crate::component_system::components::utils::commands::ToCommand;
use crate::component_system::{self as cmp, FrameworkConfig, command_parser as cmd};
use super::utils;
use super::utils::message;
use super::utils::Data;
use super::utils::commands;

macro_rules! err_println {
    (send_error($ctx: ident, $msg: ident, $txt:expr)) => {
        err_println!(utils::send::error_message($ctx, $msg, $txt).await, "Error sending error message: {}")
    };
    (send_success($ctx: ident, $msg: ident, $txt:expr)) => {
        err_println!(utils::send::success_message($ctx, $msg, $txt).await, "Error sending success message: {}")
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

/// Le composant de gestion des tickets
pub struct Tickets {
    /// Données persistantes
    data: RwLock<Data<DataTickets>>,
    /// Configuration des commandes
    node: cmd::Node,
    /// Dossier de sauvegarde des tickets
    /// 
    /// Dès que les tickets sont supprimés, ils sont enregistrés dans ce dossier.
    archives_folder: PathBuf
}
/// Données persistantes du composant
/// 
/// A chaque écriture dans le fichier de données, le fichier est sauvegardé
#[derive(Serialize, Deserialize, Default, Debug)]
struct DataTickets {
    /// Identifiants du channel et du message pour choisir le type de ticket
    /// Ces identifiants est enregistré pour pouvoir le remplacer si nécessaire
    msg_choose: Option<(u64, u64)>,
    /// [Catégories] de tickets
    /// 
    /// [Catégories]: CategoryTicket
    categories: Vec<CategoryTicket>,
}

/// Catégorie de tickets
#[derive(Serialize, Deserialize, Default, Debug)]
struct CategoryTicket {
    /// Nom de la catégorie
    name: String, 
    /// Préfix de ticket
    /// 
    /// Le préfix est utilisé pour créer le titre d'un ticket tel que 
    /// `<prefix>_<username>`
    prefix: String,
    /// Identifiant de la catégorie Discord
    id: u64,
    /// Description de la catégorie
    desc: Option<String>,
    /// Emoji décorateur de la catégorie
    /// 
    /// Pas encore en place
    emoji: Option<ReactionType>,
    /// Tickets créés dans cette catégorie
    tickets: Vec<String>,
}




#[async_trait]
impl crate::component_system::Component for Tickets {
    fn name(&self) -> &str {
        "tickets"
    }
    async fn command(&self, fw_config: &FrameworkConfig, ctx: &Context, msg: &Message) -> cmp::CommandMatch {
        self.r_command(fw_config, ctx, msg).await
    }
    async fn event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.node)
    }
}

impl Tickets {
    pub fn new() -> Self {
        use serenity::model::interactions::application_command::ApplicationCommandOptionType;

        let node = cmd::Node::new().add_group(
            cmd::Group::new("tickets")
                .set_help("Gestion des tickets")
                .set_permission("owners")
                .add_group(cmd::Group::new("channel")
                    .set_help("Salon de création de tickets")
                    .add_command(cmd::Command::new("set")
                        .set_help("Change le salon")
                        .add_param(cmd::Argument::new("id")
                            .set_value_type(ApplicationCommandOptionType::Channel)
                            .set_required(true)
                            .set_help("Identifiant du message")
                        )
                    )
                )
                .add_command(cmd::Command::new("close")
                    .set_help("Ferme le salon")
                )
                .add_group(cmd::Group::new("categories")
                    .set_help("Gestion des catégories de tickets.")
                    .add_command(cmd::Command::new("add")
                        .set_help("Ajoute une catégorie de ticket. À ne pas confondre avec les catégories discord")
                        .add_param(cmd::Argument::new("name")
                            .set_value_type(ApplicationCommandOptionType::String)
                            .set_required(true)
                            .set_help("Nom de la catégorie")
                        )
                        .add_param(cmd::Argument::new("id")
                            .set_value_type(ApplicationCommandOptionType::Channel)
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
                    )
                    .add_command(cmd::Command::new("remove")
                        .set_help("Supprime une catégorie de ticket")
                        .add_param(cmd::Argument::new("name")
                            .set_required(true)
                            .set_help("Nom de la catégorie")
                        )
                    )
                    .add_command(cmd::Command::new("list")
                        .set_help("Liste les catégories de ticket")
                    )
                )
                
            );
        Tickets{
            node,
            data: match Data::from_file_default("tickets") {
                Ok(data) => RwLock::new(data),
                Err(e) => panic!("Data tickets: {:?}", e)
            },
            archives_folder: utils::DATA_DIR.join("archives"),
        }
    }
    /// Execute les commandes du composant __non slash__
    async fn r_command(&self, _: &FrameworkConfig, ctx: &Context, msg: &Message) -> cmp::CommandMatch {
        let args = cmd::split_shell(&msg.content[1..]);
        
        let matched = match utils::try_match(ctx, msg, &self.node, args).await {
            Ok(v) => v,
            Err(e) => return e
        };
        println!("matched: {:?}", matched);
        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return cmp::CommandMatch::Error("Cette commande n'est pas disponible en message privé.".to_string())
        };
        let command = matched.to_command();
        println!("{:?}", command);
        let c_msg = match self.commands(ctx, guild_id, command).await {
            Ok(v) => v,
            Err(Some(e)) => return cmp::CommandMatch::Error(e),
            Err(None) => return cmp::CommandMatch::NotMatched,
        };
        match utils::send::custom(ctx, msg.channel_id, c_msg).await {
            Ok(_) => cmp::CommandMatch::Matched,
            Err(e) => cmp::CommandMatch::Error(format!("Erreur lors de l'envoi du message de réponse: {}", e.to_string()))
        }
    }
    /// Dispatch un enevement reçu par le bot
    async fn r_event(&self, ctx: &Context, evt: &Event) -> Result<(), String> {
        use serenity::model::event::Event::*;

        match evt {
            Ready(_) => self.on_ready(ctx).await,
            InteractionCreate(evt) => self.on_interaction(ctx, &evt.interaction).await.or_else(|e| {
                eprintln!("Erreur lors de la création d'une interaction: {}", e);
                Err(e)
            }),
            _ => Ok(())
        } 
    }
    /// Dispatch une intération reçue par le bot
    async fn on_interaction(&self, ctx: &Context, interaction: &Interaction) -> Result<(), String> {
        match interaction {
            Interaction::Ping(_) => Ok(()),
            Interaction::ApplicationCommand(v) => self.on_app_command(ctx, v).await,
            Interaction::MessageComponent(v) => self.on_msg_component(ctx, v).await,
        }
    }
    async fn commands(&self, ctx: &Context, guild_id: GuildId, command: commands::Command, app_cmd: &utils::app_command::ApplicationCommandEmbed<'_>) -> Result<Option<message::Message>, Option<String>> {
        match command.fullname().as_str() {
            "tickets.channel.set" => {
                let channel = match command.get_argument("id") {
                    Some(&commands::Argument{value: commands::Value::Channel(v), ..}) => v,
                    Some(_) => return Err(Some("id: paramètre mal formé".to_string())),
                    None => return Err(Some("id: paramètre manquant".to_string()))
                };
                Ok(Some(self.set_channel(ctx, guild_id, channel.0).await))
            },
            "tickets.close" => {
                match self.close_channel(ctx, guild_id, app_cmd.0.channel_id).await {
                    Ok(_) => Ok(None),
                    Err(e) => Err(Some(e))
                }
            },
            "tickets.categories.add" => {
                let name = match command.get_argument("name") {
                    Some(&commands::Argument{value: commands::Value::String(ref v), ..}) => v.clone(),
                    Some(_) => return Err(Some("name: paramètre mal formé".to_string())),
                    None => return Err(Some("name: paramètre manquant".to_string()))
                };
                let prefix = match command.get_argument("prefix") {
                    Some(&commands::Argument{value: commands::Value::String(ref v), ..}) => v.clone(),
                    Some(_) => return Err(Some("prefix: paramètre mal formé".to_string())),
                    None => return Err(Some("prefix: paramètre manquant".to_string()))
                };
                let desc = match command.get_argument("desc") {
                    Some(&commands::Argument{value: commands::Value::String(ref v), ..}) => Some(v.clone()),
                    Some(_) => return Err(Some("desc: paramètre mal formé".to_string())),
                    None => None
                };
                let channel = match command.get_argument("id") {
                    Some(&commands::Argument{value: commands::Value::Channel(v), ..}) => v,
                    Some(_) => return Err(Some("id: paramètre mal formé".to_string())),
                    None => return Err(Some("id: paramètre manquant".to_string()))
                };
                Ok(Some(self.category_add(ctx, guild_id, name, desc, channel.0, prefix).await))
            }
            "tickets.categories.remove" => {
                let name = match command.get_argument("name") {
                    Some(&commands::Argument{value: commands::Value::String(ref v), ..}) => v.clone(),
                    Some(_) => return Err(Some("name: paramètre mal formé".to_string())),
                    None => return Err(Some("name: paramètre manquant".to_string()))
                };
                Ok(Some(self.category_remove(ctx, name).await))
            },
            "tickets.categories.list" => Ok(Some(self.categories_list(ctx, guild_id).await)),
            _ => return Err(None)
        }
    }
    /// Dispatch les commandes slash reçu par le bot
    async fn on_app_command(&self, ctx: &Context, app_command: &ApplicationCommandInteraction) -> Result<(), String> {
        let app_cmd = utils::app_command::ApplicationCommandEmbed::new(app_command);
        let guild_id = match app_cmd.get_guild_id() {
            Some(v) => v,
            None => return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        let command = app_command.to_command();
        println!("{:?}", command);
        let c_msg = match self.commands(ctx, guild_id, command, &app_cmd).await {
            Ok(v) => v,
            Err(None) => return Ok(()),
            Err(Some(e)) => return Err(e)
        };
        if let Some(c_msg) = c_msg {
            app_command.create_interaction_response(ctx, |resp|{
                *resp = c_msg.into();
                resp
            }).await.or_else(|e| {
                eprintln!("Cannot create response: {}", e);
                Err(e.to_string())
            })
        } else {
            Ok(())
        }
    }
    /// Dispatch les composants de message (boutons, menu déroulant) reçu par le bot
    async fn on_msg_component(&self, ctx: &Context, msg_component: &MessageComponentInteraction) -> Result<(), String> {
        let res = match msg_component.data.custom_id.as_str() {
            "tickets_create" => self.on_ticket_create(ctx, msg_component).await,
            "tickets_close" => self.on_ticket_close(ctx, msg_component).await,
            _ => Ok(())
        };
        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                let err = format!("Error on message component: {}", e);
                eprintln!("{}", err);
                Err(err)
            }
        }
    }
    /// Méthode éxécuté sur l'evenement Ready du bot
    /// 
    /// Met à jour le menu déroulant des catégories
    async fn on_ready(&self, ctx: &Context) -> Result<(), String> {
        match self.update_select_menu(ctx).await {
            Ok(_) => (),
            Err(e) => eprintln!("ticket: Error updating message components: {}", e)
        };
        Ok(())
    }
    /// Créer un ticket
    /// 
    /// Lorsque l'on sélectionne le menu déroulant, la méthode créer un nouveau ticket 
    /// avec les permissions requise et informe l'utilisateur de la création du ticket
    async fn on_ticket_create(&self, ctx: &Context, msg_cmp: &MessageComponentInteraction) -> serenity::Result<()> {
        use serenity::model::prelude::*;
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
        let new_channel = guild_id.create_channel(ctx, |ch| {
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
        }).await?;
        self.update_select_menu(ctx).await?;
        match new_channel
            .send_message(ctx, |msg|
                msg
                    .content(format!("Hey {}, par ici !\nDès que tu as fini avec le ticket, appuie sur le bouton \"Fermer le ticket\".", member.mention()))
                    .components(|cmps| {
                        self.create_close_button(cmps, "Fermer le ticket", ButtonStyle::Danger);
                        cmps
                    })
            ).await {
                Ok(msg) => new_channel.pin(ctx, msg.id).await?,
                Err(e) => eprintln!("Error sending message to new channel: {}", e)
            }
        msg_cmp.create_interaction_response(ctx, |resp| 
            resp
                .interaction_response_data(|resp_data|
                    resp_data
                        .content(format!("Le ticket a bien été créé.\n\nVous pouvez le rejoindre en cliquant sur le lien suivant: {}", new_channel.mention()))
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                )
                .kind(InteractionResponseType::ChannelMessageWithSource)
        ).await?;
        Ok(())
    }
    /// Fermer un ticket
    /// 
    /// Lorsque l'on clique sur le bouton de fermeture, la méthode [archive] puis ferme le ticket
    /// 
    /// [archive]: Self::archive_channel
    async fn on_ticket_close(&self, ctx: &Context, msg_cmp: &MessageComponentInteraction) -> serenity::Result<()> {
        match Self::archive_channel(ctx, msg_cmp.channel_id).await {
            Ok(_) => msg_cmp.channel_id.delete(ctx).await.and(Ok(()))?,
            Err(e) => eprintln!("Error archiving channel: {}", e)
        }
        Ok(())
    }
    /// Retourne le chemin du dossier de stockage des tickets
    /// 
    /// Le dossier est créé s'il n'existe pas
    fn get_archive_folder() -> Result<PathBuf, std::io::Error> {
        let path = utils::DATA_DIR.join("tickets/archives");
        if !path.exists() {
            println!("tickets: Création du dossier d'archives");
            match std::fs::create_dir_all(&path) {
                Ok(_) => println!("tickets: Dossier créé"),
                Err(e) => return Err(e)
            }
        }
        Ok(path)
    }
    /// Archive un ticket
    /// 
    /// Le nom et l'avatar des utilisateur liés au tickets sont enregistrés ainsi que leurs messages
    async fn archive_channel(ctx: &Context, channel: ChannelId) -> Result<(), String> {
        let archive_path = match Self::get_archive_folder() {
            Ok(v) => v,
            Err(_) => return Err("Impossible de créer le dossier d'archives".to_string())
        };

        let file_path = archive_path.join(format!("{}-{}.txt", channel.0, channel.name(ctx).await.unwrap()));
        let mut file = match std::fs::File::create(&file_path) {
            Ok(v) => v,
            Err(e) => return Err(format!("Impossible de créer le fichier d'archive: {}", e))
        };
        let mut users = HashMap::new();
        struct UserData {
            id: u64,
            avatar: String
        }
        let contents: Vec<String> = channel
            .messages_iter(ctx)
            .map(|v|{
                match v {
                    Ok(v) => {
                        let attachments = if v.attachments.is_empty() {
                            None
                        } else {
                            let attachs = v.attachments.iter().map(|attachment| {
                                let is_image = attachment.content_type.as_ref().map(|v| v.contains("image")).unwrap_or(false);
                                format!("\n{}[{}]({})", if is_image {"!"} else {""}, attachment.filename, attachment.url)
                            }).collect::<String>();
                            Some(format!("\nFichiers joints: {}", attachs))
                        };
                        let content = v.content.as_str();
                        let author = format!("{}#{:04}", v.author.name, v.author.discriminator);
                        let date = v.timestamp.to_rfc3339();
                        if users.get(&author).is_none() {
                            users.insert(author.clone(), UserData{
                                id: v.author.id.0,
                                avatar: v.author.avatar_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string())
                            });
                        }
                        format!("[{}] {}: {}{}\n\n", date, author, content, attachments.unwrap_or_default())
                    },
                    Err(e) => format!("Erreur lors de la récupération d'un message: {}\n", e)
                }
            }).collect().await;
        users.iter().for_each(|(username, userdata)| {
            file.write_all(format!("{}\navatar_url: {}\nid: {}\n", username, userdata.avatar, userdata.id).as_bytes()).unwrap();
        });
        contents.iter().rev().for_each(|v| {
            match file.write_all(v.as_bytes()) {
                Ok(_) => (),
                Err(e) => eprintln!("Error writing to file: {}", e)
            }
        });
        Ok(())
    }
    /// Créer un bouton de fermeture de ticket
    /// 
    /// Ce bouton ets attaché au message d'ouverture du ticket
    fn create_close_button<S: ToString>(&self, cmps: &mut serenity::builder::CreateComponents, label: S, style: ButtonStyle){
        use serenity::builder::*;
        
        let mut button = CreateButton::default();
        button.label(label.to_string())
            .custom_id("tickets_close")
            .style(style);

        let mut act = CreateActionRow::default();
        act.add_button(button);
        cmps.add_action_row(act);
    }
    /// Supprime l'ancien message de création de ticket, celui contenant le menu de sélection de catégorie
    async fn delete_old_creation_message(&self, ctx: &Context) -> serenity::Result<()> {
        let old_msg = self.data.read().await.read().msg_choose;
        if let Some((channel_id, msg_id)) = old_msg {
            ctx.http.delete_message(channel_id, msg_id).await
        } else {
            Ok(())
        }
    }
    async fn set_channel(&self, ctx: &Context, guild_id: GuildId, id: u64) -> message::Message {
        if let Err(e) =  self.delete_old_creation_message(ctx).await {
            eprintln!("tickets: unable to delete previous message.\n{:?}", e)
        }
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
                        data.msg_choose = Some((channel.0, msg_sent.id.0));
                    }
                    err_println!(self.update_select_menu(ctx).await, "tickets: unable to update message after adding a category.\n{:?}");
                    message::success(format!("Le message de création de ticket a été mis à jour dans le salon {}.", channel_name))
                },
                Err(e) => message::error(format!("{}", e.to_string()))
            }
        } else {
            message::error("Le salon n'existe pas.")
        }
    }
    /// Créer un menu de sélection de catégorie
    fn create_select_menu(&self, cmps: &mut serenity::builder::CreateComponents, categories: &Vec<CategoryTicket>) {
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
        menus.custom_id("tickets_create");

        let mut act = CreateActionRow::default();
        act.add_select_menu(menus);
        cmps.add_action_row(act);
    }
    /// Met à jour le menu de sélection de catégorie
    async fn update_select_menu(&self, ctx: &Context) -> serenity::Result<()> {
        let data = self.data.read().await;
        let data = data.read();
        let categories = &data.categories;
        
        let (chan_id, msg_id) = match data.msg_choose {
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
                self.create_select_menu(cmps, categories);
                cmps
            });

            msg
        }).await
    }
    async fn close_channel(&self, ctx: &Context, guild_id: GuildId, channel_id: ChannelId) -> Result<(), String> {
        use serenity::model::interactions::message_component::*;
        let channel = guild_id.channels(ctx).await.unwrap().into_iter().find(|channel| channel.0.0 == channel_id.0);
        let (channel, guild_channel) = match channel {
            Some(c) => c,
            None => return Err("Le salon n'existe pas.".to_string())
        };
        // L'idée est de savoir si le premier message du salon a un bouton de fermeture de ticket
        let pins = match guild_channel.pins(ctx).await {
            Ok(pins) => pins,
            Err(e) => return Err(format!("{}", e))
        };
        let first_message = match pins.last() {
            Some(pin) => pin,
            None => return Err("Le salon n'a pas de message épinglé.".to_string())
        };
        let msg_comp = match first_message.components.first() {
            Some(comp) => comp,
            None => return Err("Aucun bouton détecté sur le premier message epinglé.".to_string())
        };
        let msg_comp = match msg_comp.components.first() {
            Some(comp) => comp,
            None => return Err("Aucun bouton détecté sur le premier message epinglé.".to_string())
        };
        if let ActionRowComponent::Button(Button{ custom_id: Some(c), ..}) = msg_comp {
            if c != "tickets_close" {
                return Err("Le premier message épinglé n'a pas de bouton pour fermer le ticket.".to_string())
            }
        } else {
            return Err("Le premier message épinglé n'a pas de bouton.".to_string())
        }
        // Le channel est détecté comme un ticket, on le ferme
        match channel.delete(ctx).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Impossible de fermer le ticket: {}", e.to_string()))
        }
    }
    /// Ajoute une catégorie de ticket
    async fn category_add(&self, ctx: &Context, guild_id: GuildId, name: String, desc: Option<String>, channel_id: u64, prefix: String) -> message::Message {
        if let Some(_) = self.data.read().await.read().categories.iter().find(|v| v.name == name) {
            return message::error(format!("La catégorie de ticket {} existe déjà.", name));
        }
        let (_, guild_channel) = match guild_id.channels(ctx).await.unwrap().into_iter().find(|channel| channel.0.0 == channel_id) {
            Some(v) => v,
            None => return message::error("Le salon n'existe pas.")
        };
        match guild_channel.kind {
            serenity::model::channel::ChannelType::Category => (),
            _ => return message::error(format!("L'id ne pointe pas sur une catégorie mais sur {} de type {:?}.", guild_channel.mention().to_string(), guild_channel.kind))
        }
        {
            let mut data = self.data.write().await;
            let mut data = data.write();
            data.categories.push(CategoryTicket{
                name,
                id: channel_id,
                tickets: Vec::new(),
                desc,
                prefix,
                emoji: None,
            });
        }
        err_println!(self.update_select_menu(ctx).await, "tickets: unable to update message after adding a category.\n{:?}");
        let data = self.data.read().await;
        let category = data.read().categories.last().unwrap();
        message::success(format!("La catégorie {} a été ajoutée.", category.name))
    }
    /// Supprime une catégorie de ticket
    async fn category_remove(&self, ctx: &Context, name: String) -> message::Message {
        let i = match self.data.read().await.read().categories.iter().position(|v| v.name == name) {
            Some(i) => i,
            None => return message::error(format!("La catégorie {} n'existe pas.", name))
        };
        self.data.write().await.write().categories.swap_remove(i);
        err_println!(self.update_select_menu(ctx).await, "tickets: unable to update message after deleting a category.\n{:?}");
        message::success(format!("La catégorie {} a été supprimée.", name))
    }
    /// Liste les catégories de ticket du composant
    async fn categories_list(&self, ctx: &Context, guild_id: GuildId) -> message::Message {
        let data = self.data.read().await;
        let data = data.read();
        let categories = &data.categories;
        if categories.is_empty() {
            return message::error("Aucune catégorie de ticket n'a été créée.")
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
        let mut msg = message::Message::default();
        msg.embed = Some({
            use serenity::builder::CreateEmbed;
            let mut embed = CreateEmbed::default();
            embed
                .title("Liste des catégories")
                .description( cat.join("\n") )
                .color(0x1ed760);
            embed
        });
        msg
    }
}