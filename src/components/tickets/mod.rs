//! Ticket manager

#[cfg(feature = "migration_json_db")]
mod json_to_db;

use std::sync::Arc;
use crate::{
    log_error, log_warn, 
    db::{
        model::ticket::category, 
        controller as db_ctrl,
        IDType
    },
    config::Tickets as ConfigTicket, log_info
};
use sea_orm::EntityTrait;
use cddio_core::{message, ApplicationCommandEmbed};
use cddio_macros::component;
use serde::{Serialize, Deserialize};
use serenity::{
    client::Context,
    model::{id::*, channel::Message, event::ReadyEvent, prelude::Member},
    model::application::interaction:: {
        message_component::MessageComponentInteraction
    }, builder::CreateSelectMenuOption
};
use sea_orm::{entity::*, query::*};

use super::utils::data2::{Data, DataGuard};

/// Le composant de gestion des tickets
pub struct Tickets {
    /// Données persistantes du composant
    data: Data<DataTickets>,
    /// Configutation
    config: Option<ConfigTicket>,
    /// Connexion a la base de données
    database: Arc<sea_orm::DatabaseConnection>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
struct MessageChoice {
    message_id: u64,
    channel_id: u64
}

/// Données persistantes du composant
/// 
/// A chaque écriture dans le fichier de données, le fichier est sauvegardé
#[derive(Serialize, Deserialize, Default, Debug)]
struct DataTickets {
    /// Identifiants du channel et du message pour choisir le type de ticket
    /// Ces identifiants est enregistré pour pouvoir le remplacer si nécessaire
    message_choice: Option<MessageChoice>,
}

impl From<category::Model> for CreateSelectMenuOption {
    fn from(ticket: category::Model) -> Self {
        let mut menu_option = CreateSelectMenuOption::new(&ticket.name, &ticket.name);
        if let Some(v) = &ticket.description {
            menu_option.description(v.as_str());
        }
        menu_option
    }
}
impl From<&category::Model> for CreateSelectMenuOption {
    fn from(ticket: &category::Model) -> Self {
        let mut menu_option = CreateSelectMenuOption::new(&ticket.name, &ticket.name);
        if let Some(v) = &ticket.description {
            menu_option.description(v.as_str());
        }
        menu_option
    }
}
fn category_to_message(model: &category::Model, title: &str) -> message::Message {
    let mut msg = message::Message::new();
    let mut embed = message::Embed::default();
    embed.color(message::COLOR_INFO);
    embed.title(title);
    embed.field(&model.name, model.description.as_ref().map(|v| v.as_str()).unwrap_or("*Aucune description*"), false);
    msg.add_embed(|e| {*e=embed; e});
    msg
}

impl Tickets {
    /// Créer un nouveau composant de gestion des tickets
    pub fn new(config: Option<ConfigTicket>, database: Arc<sea_orm::DatabaseConnection>) -> Self {
        let data = Self::new_data();
        Self {
            data,
            config,
            database,
        }
    }
    fn new_data() -> Data<DataTickets> {
        Data::from_file_or_default("tickets").expect("Impossible d'importer le fichier de données")
    }
}

#[component]
#[group(name="tickets", description="Gestion des tickets")]
#[group(parent="tickets", name="categories", description="Gestion des catégories de tickets")]
#[group(name="ticket", description="Commandes dans un ticket")]
impl Tickets {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, _:&ReadyEvent) {
        #[cfg(feature = "migration_json_db")]
        self.do_migration_json_db(ctx).await;

        if let Err(e) = self.update_menu(ctx).await {
            log_error!("Erreur lors de la mise à jour du menu: {}", e);
        }
    }
    #[cfg(feature = "migration_json_db")]
    async fn do_migration_json_db(&self, ctx: &Context) {
        log_info!("Migration des données de tickets...");
        let res_migration = json_to_db::do_migration(&self.database, ctx.cache.current_user().id.0 as IDType).await;
        std::fs::write("migration.log", format!("{:#?}", res_migration)).unwrap();
        log_info!("Migration des données de tickets terminée");
    }
    #[command(group="tickets", description="Assigne le salon de création de tickets")]
    async fn set_channel(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="salon", description="Salon textuel")]
        chan: Option<ChannelId>
    ) {
        let resp = match app_cmd.delayed_response(ctx, true).await {
            Ok(resp) => resp,
            Err(err) => {
                log_error!("Erreur lors de la création de la réponse: {}", err);
                return;
            }
        };
        'msg: {
            let message_choice = self.data.read().await.message_choice;
            if let Some(MessageChoice { channel_id, message_id }) = message_choice {
                let msg = match ChannelId(channel_id).message(ctx, message_id).await {
                    Ok(msg) => msg,
                    Err(err) => {
                        log_warn!("Erreur lors de la récupération du menu: {}", err);
                        break 'msg;
                    }
                };
                if let Err(err) = msg.delete(ctx).await {
                    log_warn!("Erreur lors de la récupération du message: {}", err);
                    break 'msg;
                }
            }
        }
        let channel = chan.unwrap_or(app_cmd.0.channel_id);

        let msg = match channel.send_message(ctx, |msg| msg.content("Sélectionnez le type de ticket que vous souhaitez créer :")).await {
            Ok(msg) => msg,
            Err(err) => {
                log_error!("Erreur lors de l'envoi du message: {:?}", err);
                return;
            }
        };
        self.data.write().await.message_choice = Some(MessageChoice{channel_id: channel.0, message_id: msg.id.0});
        self.update_menu(ctx).await.unwrap_or_else(|e| {
            log_error!("Erreur lors de la mise a jour du menu: {:?}", e);
        });
        
        if let Err(err) = resp.send_message(message::success("Salon de création de tickets configuré")).await {
            log_error!("Erreur lors de l'envoi de la réponse: {:?}", err);
        }
    }
    #[command(group="ticket", name="close", description="Ferme le ticket actuel")]
    async fn ticket_close(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        if let Err(e) = self.ticket_close_channel(ctx, app_cmd.0.channel_id, app_cmd.0.member.as_ref()).await {
            Self::send_error(ctx, app_cmd, e).await;
        }
    }
    #[command(group="categories", name="add", description="Ajoute une catégorie de ticket. À ne pas confondre avec les catégories discord")]
    async fn add_categorie(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="nom", description="Nom de la catégorie")]
        name: String,
        #[argument(description="Catégorie Discord où les tickets seront créés", name="categorie_discord")]
        category_id: ChannelId,
        #[argument(description="Préfixe des tickets", name="prefix")]
        prefix: String,
        #[argument(description="Cacher la catégorie du menu de ticket ?")]
        hidden: bool,
        #[argument(description="Description de la catégorie", name="description")]
        desc: Option<String>
    ) {
        let res = 'error: {
            let nb_categories = category::Entity::find()
                .filter(category::Column::Name.eq(&name))
                .count(&*self.database).await;
            match nb_categories {
                Ok(nb) if nb > 0 => break 'error Err("Cette catégorie existe déjà".to_string()),
                Err(err) => break 'error Err(format!("Erreur lors de la récupération du nombre de catégories: {}", err)),
                _ => ()
            }
            let category_id = match db_ctrl::ticket::add_category(&*self.database, name, prefix, category_id, desc, Some(hidden)).await {
                Ok(id) => id,
                Err(err) => break 'error Err(format!("Erreur lors de la création de la catégorie dans la base de données: {}", err))
            };
            if let Err(e) = self.update_menu(ctx).await {
                break 'error Err(format!("Erreur lors de la mise à jour du menu: {}", e));
            }
            let category_model = match category::Entity::find_by_id(category_id).one(&*self.database).await {
                Ok(Some(model)) => model,
                Ok(None) => break 'error Err("L'insertion de la catégorie dans la base de données a échoué".to_string()),
                Err(err) => break 'error Err(format!("Erreur lors de la récupération de la catégorie dans la base de données: {}", err))
            };
            let msg = category_to_message(&category_model, "Catégorie créée");
            app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
                log_error!("Erreur lors de l'envoi du message: {}", e);
            });
            Ok(())
        };
        if let Err(e) = res {
            Self::send_error(ctx, app_cmd, e).await;
        }
    }
    #[command(group="categories", name="remove", description="Supprime une catégorie de ticket")]
    async fn remove_categorie(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="nom", description="Nom de la catégorie")]
        name: String
    ) {
        let res = 'error: {
            let cat = category::Entity::find()
                .filter(category::Column::Name.eq(name))
                .columns([category::Column::Id, category::Column::Name, category::Column::Description].into_iter())
                .one(&*self.database).await;
            let cat = match cat {
                Ok(Some(cat)) => cat,
                Ok(None) => break 'error Err("Cette catégorie n'existe pas".to_string()),
                Err(err) => break 'error Err(format!("Erreur lors de la récupération de la catégorie dans la base de données: {:#?}", err))
            };
            if let Err(e) = db_ctrl::ticket::remove_category(&*self.database, cat.id).await {
                break 'error Err(format!("Erreur lors de la suppression de la catégorie dans la base de données: {:#?}", e));
            }
            if let Err(e) = self.update_menu(ctx).await {
                break 'error Err(format!("Erreur lors de la mise à jour du menu: {}", e));
            }
            Ok(cat)
        };
        match res {
            Ok(cat) => {
                let msg = category_to_message(&cat, "Catégorie supprimée");
                app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
                    log_error!("Erreur lors de l'envoi du message: {}", e);
                });
            }
            Err(e) => Self::send_error(ctx, app_cmd, e).await
        }
    }
    #[command(group="categories", name="list", description="Liste les catégories de ticket")]
    async fn list_categories(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        let res = 'error: {
            let categories = match category::Entity::find().all(&*self.database).await {
                Ok(categories) => categories,
                Err(err) => break 'error Err(format!("Erreur lors de la récupération des catégories dans la base de données: {:#?}", err))
            };
            let mut msg = message::Message::new();
            let mut embed = message::Embed::default();
            embed.title("Liste des catégories");
            embed.color(message::COLOR_INFO);
            for cat in categories {
                embed.field(&cat.name, cat.description.clone().unwrap_or_else(|| "*Aucune desscription*".into()), false);
            }
            msg.add_embed(|e| {*e=embed; e});
            app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
                log_error!("Erreur lors de l'envoi du message: {}", e);
            });
            Ok(())
        };
        if let Err(e) = res {
            Self::send_error(ctx, app_cmd, e).await
        }
    }
    #[command(group="ticket", description="Ajoute une personne au ticket")]
    async fn add_member(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="qui", description="Personne à ajouter au ticket")]
        personne: UserId
    ) {
        use serenity::model::{
            channel::{PermissionOverwrite, PermissionOverwriteType},
            permissions::Permissions,
        };
        let channel_id = app_cmd.0.channel_id;
        let delay_resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("Erreur lors de l'envoi du message: {}", e);
                return;
            }
        };
        let msg = 'msg: {
            let guild_id = match app_cmd.0.guild_id {
                Some(guild_id) => guild_id,
                None => break 'msg message::error("Cette commande n'est pas disponible dans un DM"),
            };
            
            match self.is_a_ticket(ctx, channel_id).await  {
                Ok(true) => (),
                Ok(false) => break 'msg message::error("Ce salon n'est pas un ticket"),
                Err(e) => break 'msg message::error(e),
            }
            let is_staff = match Self::is_staff(ctx, guild_id, app_cmd.0.user.id).await {
                Ok(v) => v,
                Err(e) => break 'msg message::error(e),
            };
            let is_owner = match Self::is_ticket_owner(ctx, channel_id, app_cmd.0.user.id).await {
                Ok(v) => v,
                Err(e) => break 'msg message::error(e),
            };
            if !is_staff && !is_owner {
                break 'msg message::error("Vous n'avez pas la permission d'ajouter des membres au ticket.");
            }
            
            let username = personne.to_user(ctx).await.map(|u| super::utils::user_fullname(&u)).unwrap_or_else(|_| personne.0.to_string());
            match channel_id.create_permission(ctx, &PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Default::default(),
                kind: PermissionOverwriteType::Member(personne),
            }).await {
                Ok(_) => message::success(format!("{} a bien été ajoutée.", username)),
                Err(e) => message::error(format!("Impossible d'ajouter {}: {}", personne, e.to_string()))
            }
        };
        delay_resp.send_message(msg).await.unwrap_or_else(|e| {
            log_error!("Erreur lors de l'envoi du message: {}", e);
        });
    }
    #[message_component(custom_id="menu_ticket_create")]
    async fn on_menu_ticket_create(&self, ctx: &Context, msg: &MessageComponentInteraction) {
        use serenity::model::application::interaction::InteractionResponseType;
        let ok = match msg.create_interaction_response(ctx, |resp| {
            resp.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.ephemeral(true)
                })
        }).await {
            Ok(_) => true,
            Err(e) => {
                log_warn!("Erreur lors de la création de l'interaction: {}", e);
                false
            }
        };
        if let Err(e) = self.update_menu(ctx).await {
            log_error!("Erreur lors de la mise à jour du menu: {}", e);
        }
        
        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id,
            None => {
                log_error!("Le menu n'est pas dans un serveur");
                return;
            }
        };
        let user_id = msg.user.id;
        let category = {
            let category_name = match msg.data.values.iter().next() {
                Some(value) => value.clone(),
                None => {
                    log_error!("Aucun item n'a été sélectionné");
                    return;
                }
            };
            match category::Entity::find().filter(category::Column::Name.eq(&category_name)).one(self.database.as_ref()).await {
                Ok(Some(category)) => category,
                Ok(None) => {
                    log_error!("La catégorie {} n'existe pas", category_name);
                    return;
                }
                Err(e) => {
                    log_error!("Erreur lors de la récupération de la catégorie dans la base de données: {}", e);
                    return;
                }
            }
        };
        let result = match self.ticket_create(ctx, guild_id, user_id, category).await {
            Ok(result) => message::success(format!("Ticket créé: <#{}>", result)),
            Err(e) => {
                log_error!("Erreur lors de la création du ticket: {}", e);
                message::error(e)
            }
        };
        if ok {
            match msg.edit_original_interaction_response(ctx, |resp| {
                *resp = result.into();
                resp
            }).await {
                Ok(_) => (),
                Err(e) => {
                    log_error!("Erreur lors de la modification de l'interaction: {}", e);
                }
            }
        }
        
    }
    #[message_component(custom_id="button_ticket_close")]
    async fn on_button_ticket_close(&self, ctx: &Context, msg: &MessageComponentInteraction) {
        if let Err(e) = self.ticket_close_channel(ctx, msg.channel_id, msg.member.as_ref()).await {
            log_error!("{}", e);
            msg.create_interaction_response(ctx, |resp|{
                resp.interaction_response_data(|inter| inter.content(e))
            }).await.unwrap_or_else(|e| {
                log_error!("Erreur lors de l'envoi d'une réponse d'interaction: {}", e);
            });
        }
    }
}

impl Tickets {
    async fn update_menu(&self, ctx: &Context) -> serenity::Result<()> {
        use std::ops::Deref;
        let message_choice = self.data.read().await.deref().message_choice;
        let mut msg = match message_choice {
            Some(MessageChoice { channel_id, message_id }) => ChannelId(channel_id).message(ctx, message_id).await?,
            _ => return Ok(()),
        };

        let categories = match category::Entity::find()
            .filter(category::Column::Hidden.eq(false))
            .all(&*self.database).await 
        {
            Ok(categories) => categories,
            Err(_e) => {
                todo!()
            }
        };
        let options = categories.into_iter().map(|cat| cat.into()).collect::<Vec<_>>();
        msg.edit(ctx, |msg|{
            msg.components(|comp| {
                comp.create_action_row(|action| {
                    action.create_select_menu(|menu| {
                        menu.options(|opts|{
                            opts.set_options(options)
                        }).custom_id("menu_ticket_create")
                    })
                })
            })
        }).await
    }
    async fn send_error<D: std::fmt::Display>(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>, error: D) {
        log_error!("{}", error);
        let mut msg = message::Message::new();
        let mut embed = message::Embed::default();
        embed.color(message::COLOR_ERROR);
        embed.title("Erreur");
        embed.description(error);
        msg.add_embed(|e| {*e=embed; e});
        app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
            log_error!("Erreur lors de l'envoi du message: {}", e);
        });
    }
    async fn category_from_ticket(&self, ctx: &Context, channel_id: ChannelId) -> Result<category::Model, String> {
        use crate::db::model::ticket;
        let db_channel_id: IDType = match channel_id.0.try_into() {
            Ok(v) => v,
            Err(e) => return Err(format!("Erreur de conversion de l'ID du salon: {}", e)),
        };
        let category_id = 'result: {
            // #1: If ticket found, get the category from it
            let ticket = ticket::Entity::find_by_id(db_channel_id).column(ticket::Column::CategoryId).one(&*self.database).await;
            match ticket {
                Ok(Some(t)) => break 'result t.category_id,
                Err(e) => return Err(format!("Erreur lors de la récupération d'une catégorie: {}", e)),
                _ => (),
            }
            // #2: Deduce the category from the channel prefix
            'skip_prefix: {
                let channel_name = match channel_id.name(ctx).await {
                    Some(v) => v,
                    None => break 'skip_prefix,
                };
                if let Some(pos_underscore) = channel_name.find(&['_', '-']) {
                    let prefix = &channel_name[..pos_underscore];
                    match category::Entity::find().filter(category::Column::Prefix.eq(prefix)).one(&*self.database).await {
                        Ok(Some(cat)) => break 'result cat.id,
                        Err(e) => return Err(format!("Erreur lors de la récupération d'une catégorie: {}", e)),
                        _ => (),
                    }
                }
            }
            // #3: Get the the default category from the configuration
            if let Some(ConfigTicket { default_category: Some(category_name) }) = &self.config {
                match category::Entity::find().filter(category::Column::Name.eq(category_name)).column(category::Column::Id).one(&*self.database).await {
                    Ok(Some(cat)) => break 'result cat.id,
                    Err(e) => return Err(format!("Erreur lors de la récupération d'une catégorie: {}", e)),
                    _ => (),
                }
            }
            // #4: Finally, if none found, get the first category from the database
            match category::Entity::find().column(category::Column::Id).one(&*self.database).await {
                Ok(Some(cat)) => break 'result cat.id,
                Ok(None) => return Err("Aucune catégorie dans la base de données".to_string()),
                Err(e) => return Err(format!("Erreur lors de la récupération d'une catégorie: {}", e)),
            }
        };
        match category::Entity::find_by_id(category_id).one(&*self.database).await {
            Ok(Some(cat)) => Ok(cat),
            Ok(None) => unreachable!("already checked"),
            Err(e) => Err(format!("Erreur lors de la récupération d'une catégorie: {}", e))
        }
    }
    async fn ticket_close_channel(&self, ctx: &Context, channel_id: ChannelId, member: Option<&Member>) -> Result<(), String> {
        match self.is_a_ticket(ctx, channel_id).await {
            Ok(true) => (),
            Ok(false) => return Err("Ce n'est pas un ticket".to_string()),
            Err(e) => return Err(e),
        }
        let closed_by = match member {
            Some(member) => member.user.id,
            None => ctx.cache.current_user().id,
        };
        if !db_ctrl::ticket::is_ticket_exists(&*self.database, channel_id).await.map_err(|e| format!("Erreur de la base de données: {}", e))? {
            let category = self.category_from_ticket(ctx, channel_id).await?;
            db_ctrl::ticket::create_ticket(ctx, &*self.database, category, channel_id, ctx.cache.current_user_id()).await
                .map_err(|e| format!("Erreur lors de la création du ticket: {}", e))?;
        }
        db_ctrl::ticket::archive_ticket(&*self.database, ctx, channel_id, closed_by).await
            .map_err(|e| format!("Erreur lors de l'archivage du ticket: {}", e))?;
        channel_id.delete(ctx).await
            .map_err(|e| format!("Erreur lors de la suppression du salon discord lié au ticket: {}", e))?;
        Ok(())
    }
    async fn is_a_ticket(&self, ctx: &Context, channel_id: ChannelId) -> Result<bool, String> {
        use serenity::model::channel::Channel;
        let current_channel = match channel_id.to_channel(ctx).await {
            Ok(Channel::Guild(chan)) => chan,
            Ok(_) => return Ok(false),
            Err(e) => return Err(format!("Une erreur s'est produite lors de la récupération du channel: {}", e)),
        };
        let parent_channel = match current_channel.parent_id {
            Some(id) => id,
            None => return Ok(false),
        };
        let res = category::Entity::find()
            .filter(category::Column::DiscordCategoryId.eq(parent_channel.0))
            .count(&*self.database).await;
        match res {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(format!("{}", e))
        }
    }
    async fn is_ticket_owner(ctx: &Context, channel: ChannelId, user_by: UserId) -> Result<bool, String> {
        let pins = match channel.pins(ctx).await {
            Ok(pins) => pins,
            Err(e) => return Err(format!("{}", e))
        };
        let first_message = match pins.last() {
            Some(pin) => pin,
            None => return Ok(false)
        };
        Ok(first_message.mentions.iter().find(|m| m.id == user_by).is_some())
    }
    async fn is_staff(ctx: &Context, guild_id: GuildId, user_by: UserId) -> Result<bool, String> {
        let roles = match guild_id.roles(ctx).await {
            Ok(roles) => roles,
            Err(e) => return Err(format!("{}", e))
        };
        let staff_role = match roles.into_iter().find(|role| role.1.name == "staff") {
            Some(role) => role,
            None => return Err("Le rôle 'staff' n'existe pas.".to_string())
        };
        let member = match guild_id.member(ctx, user_by).await {
            Ok(member) => member,
            Err(e) => return Err(format!("{}", e))
        };
        Ok(member.roles.into_iter().find(|role| role == &staff_role.0).is_some())
    }
    async fn reset_message_choose(&self, new_ids: Option<MessageChoice>) {
        self.data.write().await.message_choice = new_ids;
    }
    async fn ticket_create(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, category: category::Model) -> Result<ChannelId, String> {
        use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType, ChannelType};
        use serenity::model::permissions::Permissions;
        use serenity::model::application::component::ButtonStyle;
        let role_staff = match guild_id.roles(ctx).await {
            Ok(roles) => {
                let role = roles.iter().find(|(_, role)| role.name == "staff");
                match role {
                    Some((role_id, _)) => *role_id,
                    None => {
                        log_error!("Une erreur s'est produite lors de la création du ticket: Le role 'staff' n'existe pas.");
                        return Err("Une erreur s'est produite lors de la création du ticket.".to_string());
                    }
                }
            },
            Err(e) => return Err(format!("Erreur lors de la récupération des roles: {}", e))
        };
        let everyone = RoleId(guild_id.0);
        
        let permissions = vec![
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::default(),
                kind: PermissionOverwriteType::Member(user_id),
            },
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::default(),
                kind: PermissionOverwriteType::Role(role_staff),
            },
            PermissionOverwrite {
                allow: Permissions::default(),
                deny: Permissions::VIEW_CHANNEL,
                kind: PermissionOverwriteType::Role(everyone),
            },
        ];
        let username = match user_id.to_user(ctx).await {
            Ok(user) => user.name,
            Err(_) => user_id.to_string()
        };
        let new_channel = match guild_id.create_channel(ctx, |chan| {
            chan
                .name(format!("{}-{}", category.prefix, username))
                .kind(ChannelType::Text)
                .category(category.discord_category_id as u64)
                .permissions(permissions)
        }).await {
            Ok(chan) => chan,
            Err(e) => return Err(format!("Erreur lors de la création du ticket: {}", e))
        };
        let mut msg_prez = match new_channel.say(ctx, format!("Hey <@{}>, par ici !\nDès que tu as fini avec le ticket, appuie sur le bouton \"Fermer le ticket\".", user_id.0)).await {
            Ok(msg) => msg,
            Err(e) => return Err(format!("Erreur pendent l'envoi du message de presentation: {}\nLe salon a tout de même été créé: <#{}>", e, new_channel.id.0))
        };
        msg_prez.edit(ctx, |msg| {
            msg.components(|cmps| {
                cmps.create_action_row(|action|{
                    action.create_button(|button|{
                        button
                            .label("Fermer le ticket")
                            .style(ButtonStyle::Danger)
                            .custom_id("button_ticket_close")
                    })
                })
            })
        }).await.unwrap_or_else(|e| {
            log_warn!("Erreur lors de la mise en place du bouton du message de présentation: {}", e);
        });
        
        msg_prez.pin(ctx).await.unwrap_or_else(|e| {
            log_warn!("Erreur lors du pin du message de présentation: {}", e);
        });
        db_ctrl::ticket::create_ticket(ctx, &self.database, category, new_channel.id, user_id).await
            .map_err(|e| format!("Erreur lors de la création du ticket: {}", e))?;

        Ok(new_channel.id)
    }
}