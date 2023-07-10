//! Ticket manager

mod archive;

use std::sync::Arc;
use crate::{
    log_error, log_warn, 
    db::model::ticket::category
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

use super::utils::data2::Data;

/// Le composant de gestion des tickets
pub struct Tickets {
    /// Données persistantes du composant
    data: Data<DataTickets>,
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

impl From<&category::Model> for CreateSelectMenuOption {
    fn from(ticket: &category::Model) -> Self {
        let mut menu_option = CreateSelectMenuOption::new(&ticket.name, &ticket.name);
        menu_option
            .description(ticket.description.unwrap_or_default());
        menu_option
    }
}
fn category_to_message(model: &crate::db::model::ticket::category::Model, title: &str) -> message::Message {
    let mut msg = message::Message::new();
    let mut embed = message::Embed::default();
    embed.color(message::COLOR_INFO);
    embed.title(title);
    embed.field(model.name, model.description.as_ref().map(|v| v.as_str()).unwrap_or("*Aucune description*"), false);
    msg.add_embed(|e| {*e=embed; e});
    msg
}

impl Tickets {
    /// Créer un nouveau composant de gestion des tickets
    pub fn new(database: Arc<sea_orm::DatabaseConnection>) -> Self {
        let data = Data::from_file_or_default("tickets").expect("Impossible d'importer le fichier de données");
        Self {
            data,
            database,
        }
    }
}

#[component]
#[group(name="tickets", description="Gestion des tickets")]
#[group(parent="tickets", name="categories", description="Gestion des catégories de tickets")]
#[group(name="ticket", description="Commandes dans un ticket")]
impl Tickets {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, _:&ReadyEvent) {
        let message_choice = self.data.read().await.message_choice;
        if let Some(MessageChoice { channel_id, message_id }) = message_choice {
            let mut msg = match ChannelId(channel_id).message(ctx, message_id).await {
                Ok(msg) => msg,
                Err(err) => {
                    log_warn!("Erreur lors de la récupération du message du menu: {:?}", err);
                    self.reset_message_choose(None).await;
                    return;
                }
            };
            if let Err(err) = self.update_menu(ctx, &mut msg).await {
                log_warn!("Erreur lors de la mise à jour du menu: {}", err);
                self.reset_message_choose(None).await;
            }
        }
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
        loop {
            let message_choice = self.data.read().await.message_choice;
            if let Some(MessageChoice { channel_id, message_id }) = message_choice {
                let msg = match ChannelId(channel_id).message(ctx, message_id).await {
                    Ok(msg) => msg,
                    Err(err) => {
                        log_warn!("Erreur lors de la récupération du menu: {}", err);
                        break;
                    }
                };
                if let Err(err) = msg.delete(ctx).await {
                    log_warn!("Erreur lors de la récupération du message: {}", err);
                    break;
                }
            }
            break;
        }
        let channel = chan.unwrap_or(app_cmd.0.channel_id);

        let mut msg = match channel.send_message(ctx, |msg| msg.content("Sélectionnez le type de ticket que vous souhaitez créer :")).await {
            Ok(msg) => msg,
            Err(err) => {
                log_error!("Erreur lors de l'envoi du message: {:?}", err);
                return;
            }
        };
        self.update_menu(ctx, &mut msg).await.unwrap_or_else(|e| {
            log_error!("Erreur lors de la mise a jour du menu: {:?}", e);
        });
        self.data.write().await.message_choice = Some(MessageChoice{channel_id: channel.0, message_id: msg.id.0});
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
        use crate::db::model::ticket::category;
        {
            let nb_categories = category::Entity::find()
                .filter(category::Column::Name.eq(&name))
                .count(&*self.database).await;
            if let Some(display) = match nb_categories {
                Ok(nb) if nb > 0 => Some("Cette catégorie existe déjà".to_string()),
                Err(err) => Some(format!("Erreur lors de la récupération du nombre de catégories: {}", err)),
                _ => None
            } {
                Self::send_error(ctx, app_cmd, display).await;
                return;
            }
        }
        let category_id = match crate::db::controller::ticket::add_category(&*self.database, name, prefix, category_id, desc, Some(hidden)).await {
            Ok(id) => id,
            Err(err) => {
                Self::send_error(ctx, app_cmd, err).await;
                return;
            }
        };
        let category_model = match category::Entity::find_by_id(category_id).one(&*self.database).await {
            Ok(Some(model)) => model,
            Ok(None) => {
                Self::send_error(ctx, app_cmd, "L'insertion de la catégorie dans la base de données a échoué").await;
                return;
            }
            Err(err) => {
                Self::send_error(ctx, app_cmd, format!("Erreur lors de la récupération de la catégorie dans la base de données: {:#?}", err)).await;
                return;
            }
        };
        let msg = category_to_message(&category_model, "Catégorie créée");
        app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
            log_error!("Erreur lors de l'envoi du message: {}", e);
        });
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
            if let Err(e) = crate::db::controller::ticket::remove_category(&*self.database, cat.id).await {
                break 'error Err(format!("Erreur lors de la suppression de la catégorie dans la base de données: {:#?}", e));
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
        let msg = loop {
            let guild_id = match app_cmd.0.guild_id {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande n'est pas disponible dans un DM"),
            };
            
            match self.is_a_ticket(ctx, channel_id).await  {
                Ok(true) => (),
                Ok(false) => break message::error("Ce salon n'est pas un ticket"),
                Err(e) => break message::error(e),
            }
            let is_staff = match Self::is_staff(ctx, guild_id, app_cmd.0.user.id).await {
                Ok(true) => true,
                Ok(false) => false,
                Err(e) => break message::error(e),
            };
            let is_owner = match Self::is_ticket_owner(ctx, channel_id, app_cmd.0.user.id).await {
                Ok(true) => true,
                Ok(false) => false,
                Err(e) => break message::error(e),
            };
            if !is_staff && !is_owner {
                break message::error("Vous n'avez pas la permission d'ajouter des membres au ticket.");
            }
            
            let username = personne.to_user(ctx).await.map(|u| super::utils::user_fullname(&u)).unwrap_or_else(|_| personne.0.to_string());
            break match channel_id.create_permission(ctx, &PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Default::default(),
                kind: PermissionOverwriteType::Member(personne),
            }).await {
                Ok(_) => message::success(format!("{} a bien été ajoutée.", username)),
                Err(e) => message::error(format!("Impossible d'ajouter {}: {}", personne, e.to_string()))
            };
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
            match crate::db::model::ticket::category::Entity::find().filter(crate::db::model::ticket::category::Column::Name.eq(&category_name)).one(self.database.as_ref()).await {
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
    async fn update_menu(&self, ctx: &Context, msg: &mut Message) -> serenity::Result<()>{
        let categories = match category::Entity::find().all(&*self.database).await {
            Ok(categories) => categories,
            Err(_e) => {
                todo!()
            }
        };
        let options = categories.iter().filter(|cat| !cat.hidden).map(|cat| cat.into()).collect::<Vec<_>>();
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
    async fn ticket_close_channel(&self, ctx: &Context, channel_id: ChannelId, member: Option<&Member>) -> Result<(), String> {
        match self.is_a_ticket(ctx, channel_id).await {
            Ok(true) => (),
            Ok(false) => return Err("Ce n'est pas un ticket".to_string()),
            Err(e) => return Err(e),
        }
        //TODO: vérifier si le ticket existe
        let closed_by = match member {
            Some(member) => member.user.id,
            None => ctx.cache.current_user().id,
        };
        //TODO: ajouter la catégorie dans les parametres de la fonction. utiliser la catégorie par défaut si aucune
        let default_category = crate::db::model::ticket::category::Entity::find()
            .filter(crate::db::model::ticket::category::Column::Name.eq("Tickets"))
            .one(self.database.as_ref())
            .await
            .map_err(|e| format!("Erreur lors de la récupération des catégories: {}", e))?
            .map(|cat| cat.id);
        crate::db::controller::ticket::archive_ticket(self.database.as_ref(), ctx, channel_id, closed_by, default_category).await
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
            .filter(category::Column::Id.eq(parent_channel.0))
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
    async fn ticket_create(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, category: crate::db::model::ticket::category::Model) -> Result<ChannelId, String> {
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
                .category(category.discord_category_id)
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
        crate::db::controller::ticket::create_ticket(&self.database, category, new_channel.id, user_id).await
            .map_err(|e| format!("Erreur lors de la création du ticket: {}", e))?;

        Ok(new_channel.id)
    }
}