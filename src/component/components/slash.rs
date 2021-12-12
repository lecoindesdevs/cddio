use std::sync::Arc;

use futures_locks::RwLock;
use serenity::{async_trait, builder::CreateApplicationCommands, client::Context, http::CacheHttp, model::{event::InteractionCreateEvent, id::{ApplicationId, GuildId, UserId}, interactions::application_command::{ApplicationCommand, ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOption, ApplicationCommandPermissionData, ApplicationCommandPermissionType}}};
use crate::component::{self as cmp, command_parser::{self as cmd, Named}, components::utils::{self, app_command::{ApplicationCommandEmbed, get_argument}}, manager::{ArcManager}};
use super::utils::message;
use crate::component::slash;


/// Composant de gestion des commandes de l'application.
/// 
/// S'occupe d'assigner les slashs commandes à discord et de gérer leur permissions.
/// 
/// Au lancement du bot, le composant parcours les différents composant du bot et 
/// génère les slashs commandes associés en se reposant sur notre API de *command parser* 
/// pour les envoyer à Discord.
pub struct SlashCommands {
    manager: ArcManager,
    owners: Vec<UserId>,
    group_match: cmd::Node,
    commands: RwLock<Vec<(GuildId, Vec<ApplicationCommand>)>>,
    app_id: ApplicationId,
}
#[async_trait]
impl cmp::Component for SlashCommands {
    fn name(&self) -> &'static str {
        "slash"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        cmp::CommandMatch::NotMatched
    }

    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.group_match)
    }
}

/// Helper pour la lecture des différents arguments d'une commande du group `slash`.
/// 
/// Cette macro existe pour simplifier le code et éviter de répéter le code.
macro_rules! slash_argument {
    ($app_cmd:ident, command: ($self: ident, $in_guild_id:ident, $out_opt_command: ident, $out_command_id:ident)) => {
        let $out_opt_command = match get_argument!($app_cmd, "command", String) {
            Some(v) => v,
            None => return message::error("L'identifiant de la commande est requis.")
        };
        let $out_command_id = {
            let commands = $self.commands.read().await;
            let (_, commands) = match commands.iter().find(|(g, _)| *g == $in_guild_id) {
                Some(list_commands) => list_commands,
                None => return message::error("Le serveur n'est pas reconnu.")
            };
            match commands.iter().find(|c| &c.name == $out_opt_command) {
                Some(command) => command.id,
                None => return message::error("Commande non trouvé.")
            }
        };
    };
    ($app_cmd:ident, who: $opt_name:ident) => {
        let $opt_name = match $app_cmd.get_argument("who") {
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::User(user, _)),
                ..
            }) => (user.id.0, ApplicationCommandPermissionType::User),
            Some(ApplicationCommandInteractionDataOption{
                resolved: Some(ApplicationCommandInteractionDataOptionValue::Role(role)),
                ..
            }) => (role.id.0, ApplicationCommandPermissionType::Role),
            None => return message::error("L'identifiant de l'utilisateur ou du rôle est requis."),
            _ => return message::error("L'identifiant de l'utilisateur ou du rôle n'est pas reconnu."),
        };
    };
    ($app_cmd:ident, type: $opt_name:ident) => {
        let $opt_name = match get_argument!($app_cmd, "type", String).and_then(|v| Some(v.as_str())) {
            Some("allow") => true,
            Some("deny") => false,
            Some(s) => return message::error(format!("Type: mot clé `{}` non reconnu. `allow` ou `deny` attendus.", s)),
            None => return message::error("Le type de permission est requis."), 
        };
    };
    ($app_cmd:ident, $($name:ident: $var_name:tt),+) => {
        $(
            slash_argument!($app_cmd, $name: $var_name);
        )*
    };
}

impl SlashCommands {
    pub fn new(manager: ArcManager, owners: Vec<UserId>, app_id: ApplicationId) -> Self {
        use serenity::model::interactions::application_command::ApplicationCommandOptionType;
        let autocomplete_commands = Arc::new(Vec::new());
        let arg_command = cmd::Argument::new("command")
            .set_value_type(ApplicationCommandOptionType::String)
            .set_required(true)
            .set_help("Quel commande est affecté")
            .set_autocomplete(autocomplete_commands.clone());
        let arg_who = cmd::Argument::new("who")
            .set_value_type(ApplicationCommandOptionType::Mentionable)
            .set_required(true)
            .set_help("Qui est affecté");
            
        let group_match = cmd::Node::new().add_group(
            cmd::Group::new("slash")
                .set_help("Gestion des commandes slash")
                .set_permission("owners")
                .add_group(cmd::Group::new("permissions")
                    .set_help("Gérer les permissions des commandes")
                    .add_command(cmd::Command::new("set")
                        .set_help("Autoriser ou interdire une commande à un utilisateur ou un rôle")
                        .add_param(arg_who.clone())
                        .add_param(arg_command.clone())
                        .add_param(cmd::Argument::new("type")
                            .set_value_type(ApplicationCommandOptionType::String)
                            .set_required(true)
                            .set_help(r#"Type d'autorisation. "allow" ou "deny""#)
                            .set_autocomplete(Arc::new(vec![
                                "allow".to_string(),
                                "deny".to_string()
                            ]))
                        ))
                    .add_command(cmd::Command::new("reset")
                        .set_help("Retire toutes les permissions d'une commande.")
                        .add_param(arg_command.clone())
                    )
                    .add_command(cmd::Command::new("remove")
                        .set_help("Retire la permission d'un membre ou d'un rôle à une commande.")
                        .add_param(arg_command.clone())
                        .add_param(arg_who.clone())
                    )
                    .add_command(cmd::Command::new("list")
                        .set_help("Liste les permissions des commandes sur le serveur."))
                )
            );
        SlashCommands {
            commands: RwLock::new(Vec::new()),
            group_match,
            manager,
            owners,
            app_id
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        match evt {
            cmp::Event::Ready(ready) => {
                let manager = self.manager.read().await;
                let components = manager.get_components();
                let guilds = &ready.ready.guilds;
                let mut app_commands = CreateApplicationCommands::default();
                for compo in components {
                    let compo = compo.read().await;
                    let node = match compo.node() {
                        Some(group) => group,
                        None => continue
                    };
                    let commands = if compo.name() == "slash" {
                        slash::register_root_with_perm(node, true)
                    } else {
                        slash::register_root(node)
                    };
                    commands.into_iter().for_each(|command| {
                        app_commands.add_application_command(command);
                    });
                    
                }
                let mut commands = self.commands.write().await;
                for guild in guilds {
                    let guild_id = guild.id();
                    match guild_id.set_application_commands(ctx, |v| {
                        *v = app_commands.clone();
                        v
                    }).await {
                        Ok(v) => commands.push((guild_id, v)),
                        Err(why) => {
                            let name = guild.id().name(ctx).await.unwrap_or(guild.id().to_string());
                            eprintln!("Could not set application commands for guild {}: {:?}", name, why);
                        }
                    }
                }
                println!("Slash commands setted.");
            },
            cmp::Event::InteractionCreate(InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await.unwrap_or(()),
            _ => (),
        }
        Ok(())
    }
    /// Méthode appelée sur événement de création d'une application command
    /// 
    /// Dispatch les commandes de permission aux fonctions correspondantes
    /// Voir les fonctions `slash_perms_*` pour plus de détails sur leur fonctionnement
    async fn on_applications_command(&self, ctx: &Context, app_command: &ApplicationCommandInteraction) -> Result<(), String> {
        if app_command.application_id != self.app_id {
            // La commande n'est pas destiné à ce bot
            return Ok(());
        }
        let app_cmd = ApplicationCommandEmbed::new(app_command);
        let guild_id = match app_cmd.get_guild_id() {
            Some(v) => v,
            None => return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        let command_name = app_cmd.fullname();
        let msg = match command_name.as_str() {
            "slash.permissions.set" => self.slash_perms_add(ctx, guild_id, app_cmd).await,
            "slash.permissions.remove" => self.slash_perms_remove(ctx, guild_id, app_cmd).await,
            "slash.permissions.reset" => self.slash_perms_reset(ctx, guild_id, app_cmd).await,
            "slash.permissions.list" => self.slash_perms_list(ctx, guild_id).await,
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
    /// Méthode appelée sur la commande slash.permissions.set
    /// 
    /// Ajoute une permission à une commande
    /// 
    /// # Arguments
    /// 
    /// * who: L'utilisateur ou le rôle à qui on assigne la permission
    /// * command: La commande à laquelle on assigne la permission. 
    /// Seules les commandes et groupe de premier niveau sont pris en compte.
    /// * type: Le type d'autorisation à assigner. "allow" ou "deny" attendu.
    async fn slash_perms_add<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: ApplicationCommandEmbed<'a>) -> message::Message {
        let user_id = app_cmd.0.member.as_ref().unwrap().user.id;
        if !self.owners.contains(&user_id) {
            return message::error("Cette commande est reservée aux owners");
        }
        slash_argument!(app_cmd, command: (self, guild_id, opt_command, command_id), who: opt_who, type: opt_type);
        let mut old_perms = match guild_id.get_application_command_permissions(ctx, command_id).await {
            Ok(v) => v.permissions,
            Err(_) => Vec::new()
        }.into_iter().map(|v| (v.id.0, v.kind, v.permission)).collect::<Vec<_>>();
        let updated = match old_perms.iter_mut().find(|v| v.0 == opt_who.0 && v.1 == opt_who.1) {
            Some(v) => {
                if v.2 == opt_type {
                    return message::success("La permission est déjà attribué tel quel.");
                }
                v.2 = opt_type;
                true
            },
            None => {
                old_perms.push((opt_who.0, opt_who.1, opt_type));
                false
            },
        };
        
        let result = guild_id.create_application_command_permission(ctx, command_id, |perm| {
            old_perms.iter().for_each(|p| {
                perm.create_permission(|new_perm| new_perm
                    .id(p.0)
                    .kind(p.1)
                    .permission(p.2)
                );
            });
            perm
        }).await;
        match (updated, result) {
            (true, Ok(_)) => message::success(format!("La permission de la commande `{}` a été mise a jour.", opt_command)),
            (false, Ok(_)) => message::success(format!("La permission de la commande `{}` a été ajoutée.", opt_command)),
            (_, Err(why)) => message::error(format!("La permission pour la commande {} n'a pas pu être assigné: {:?}", opt_command, why))
        }
    }
    /// Méthode appelée sur la commande slash.permissions.remove
    /// 
    /// Supprime une permission à une commande
    /// 
    /// # Arguments
    /// 
    /// * who: L'utilisateur ou le rôle qui a la permission à supprimer
    /// * command: La commande à laquelle on retire la permission.
    /// Seules les commandes et groupe de premier niveau sont pris en compte.
    async fn slash_perms_remove<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: ApplicationCommandEmbed<'a>) -> message::Message {
        let user_id = app_cmd.0.member.as_ref().unwrap().user.id;
        if !self.owners.contains(&user_id) {
            return message::error("Cette commande est reservée aux owners");
        }
        slash_argument!(app_cmd, command: (self, guild_id, opt_command, command_id), who: opt_who);
        
        let mut found = false;
        let old_perms = match guild_id.get_application_command_permissions(ctx, command_id).await {
                Ok(v) => v.permissions,
                Err(_) => Vec::new()
            }
            .into_iter()
            .filter(|v| {
                let result = v.id.0 == opt_who.0 && v.kind == opt_who.1;
                found |= result;
                !result
            })
            .map(|v| (v.id.0, v.kind, v.permission))
            .collect::<Vec<_>>();
        if !found {
            return message::error("La permission n'a pas été trouvé.");
        }
        let result = guild_id.create_application_command_permission(ctx, command_id, |perm| {
            old_perms.iter().for_each(|p| {
                perm.create_permission(|new_perm| new_perm
                    .id(p.0)
                    .kind(p.1)
                    .permission(p.2)
                );
            });
            perm
        }).await;
        match result {
            Ok(_) => message::success(format!("La permission de <@{}{}> pour la commande `{}` a été retirée.", if opt_who.1 == ApplicationCommandPermissionType::Role {"&"} else {""}, opt_who.0, opt_command)),
            Err(why) => message::error(format!("Une erreur s'est produite lors de la suppression de la permission: {:?}", why))
        }
    }
    /// Méthode appelée sur la commande slash.permissions.reset
    /// 
    /// Supprime toutes les permissions d'une commande
    /// 
    /// # Arguments
    /// 
    /// * command: La commande à laquelle on retire les permissions.
    async fn slash_perms_reset<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: ApplicationCommandEmbed<'a>) -> message::Message {
        let user_id = app_cmd.0.member.as_ref().unwrap().user.id;
        if !self.owners.contains(&user_id) {
            return message::error("Cette commande est reservée aux owners");
        }
        slash_argument!(app_cmd, command: (self, guild_id, opt_command, command_id));
        match guild_id.create_application_command_permission(ctx, command_id, |perm| perm).await {
            Ok(_) => message::success(format!("Les permissions pour la commande `{}` ont été retirées.", opt_command)),
            Err(why) => message::error(format!("Une erreur s'est produite lors de la réinitialisation des permissions: {:?}", why))
        }
    }
    /// Méthode appelée sur la commande slash.permissions.list
    /// 
    /// Affiche la liste des permissions des commandes du bot
    async fn slash_perms_list<'a>(&self, ctx: &Context, guild_id: GuildId) -> message::Message {
        let commands = match guild_id.get_application_commands(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id).collect::<Vec<_>>();
        let perms = match guild_id.get_application_commands_permissions(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id).collect::<Vec<_>>();

        let perms = perms
            .into_iter()
            .filter_map(|v| {
                match commands.iter().find(|c| c.id == v.id) {
                    Some(command) => Some((command.name.clone(), v.permissions)),
                    None => None
                }
            })
            .map(|info_perms| {
                let list_perm = info_perms.1
                    .into_iter()
                    .map(|perm| {
                        let user = match perm.kind {
                            ApplicationCommandPermissionType::User => format!("<@{}>", perm.id),
                            ApplicationCommandPermissionType::Role => format!("<@&{}>", perm.id),
                            _ => "*unknown*".to_string(),
                        };
                        let permission = match perm.permission {
                            true => "est autorisé",
                            false => "est refusé",
                        };
                        format!("{} {}.\n", user, permission)
                    })
                    .collect::<String>();
                format!("*Commande __{}__*\n\n{}", info_perms.0, list_perm)
            })
            .collect::<String>();
        message::success(perms)
    }
}