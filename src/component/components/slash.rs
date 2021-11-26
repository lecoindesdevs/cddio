use std::sync::Arc;

use futures_locks::RwLock;
use serenity::{async_trait, builder::CreateApplicationCommands, client::Context, http::CacheHttp, model::{event::InteractionCreateEvent, id::GuildId, interactions::application_command::{ApplicationCommand, ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue, ApplicationCommandOption, ApplicationCommandPermissionType}}};
use crate::component::{self as cmp, command_parser::{self as cmd, Named}, components::utils::{self, app_command::{get_argument, unwrap_argument}}, manager::{ArcManager}};

use crate::component::slash;
pub struct SlashInit {
    manager: ArcManager,
    group_match: cmd::Group,
    commands: RwLock<Vec<(GuildId, Vec<ApplicationCommand>)>>
}
#[async_trait]
impl cmp::Component for SlashInit {
    fn name(&self) -> &'static str {
        "slash"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        cmp::CommandMatch::NotMatched
    }

    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn group_parser(&self) -> Option<&cmd::Group> {
        Some(&self.group_match)
    }
}

impl SlashInit {
    pub fn new(manager: ArcManager) -> Self {
        use serenity::model::interactions::application_command::ApplicationCommandOptionType;
        let autocomplete_commands = Arc::new(Vec::new());
        let command = cmd::Command::new("")
            .set_help("Change le salon")
            .add_param(cmd::Argument::new("id")
                .set_value_type(ApplicationCommandOptionType::Mentionable)
                .set_required(true)
                .set_help("Qui est affecté")
            )
            .add_param(cmd::Argument::new("command")
                .set_value_type(ApplicationCommandOptionType::String)
                .set_required(true)
                .set_help("Quel commande est affecté")
                .set_autocomplete(autocomplete_commands.clone())
            )
            .add_param(cmd::Argument::new("type")
                .set_value_type(ApplicationCommandOptionType::String)
                .set_required(true)
                .set_help("Type d'autorisation")
                .set_autocomplete(Arc::new(vec![
                    "allow".to_string(),
                    "deny".to_string()
                ]))
            );
            
        let mut group_match = cmd::Group::new("slash")
            .set_help("Gestion des commandes slash")
            .set_permission("owners")
            .add_group(cmd::Group::new("permissions")
                .set_help("Gérer les permissions des commandes")
                .add_command({
                    let mut cmd = command.clone();
                    cmd.name = "set".into();
                    cmd
                })
                .add_command({
                    let mut cmd = command.clone();
                    cmd.name = "add".into();
                    cmd
                })
            );
        group_match.generate_ids(None);
        SlashInit {
            commands: RwLock::new(Vec::new()),
            group_match,
            manager
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
                    let group = match compo.group_parser() {
                        Some(group) => group,
                        None => continue
                    };
                    app_commands.add_application_command(slash::register_root(group));
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
    async fn on_applications_command(&self, ctx: &Context, app_cmd: &ApplicationCommandInteraction) -> Result<(), String> {
        let app_command = utils::app_command::ApplicationCommand::new(app_cmd);
        let command_name = app_command.fullname();
        let guild_id = match app_command.get_guild_id() {
            Some(v) => v,
            None => return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        if !command_name.starts_with("slash.permissions") {
            return Ok(());
        }
        let commands = self.commands.read().await;
        let (_, commands) = match commands.iter().find(|(g, _)| *g == guild_id) {
            Some(list_commands) => list_commands,
            None => return Ok(())
        };
        let opt_command = match get_argument!(app_command, "command", String) {
            Some(opt_command) => opt_command,
            None => return Ok(())
        };
        println!("{:?}", opt_command);
        let command_id = match commands.iter().find(|c| &c.name == opt_command) {
            Some(command) => command.id,
            None => return Ok(())
        };
        let opt_type = get_argument!(app_command, "type", String);
        let opt_type = match opt_type {
            Some(s) if s == "allow" => true,
            Some(s) if s == "deny" => false,
            _ => return Ok(())
        };
        println!("{:?}", opt_type);
        
        let opt_who = {
            let who = app_command.get_argument("id");
            match who {
                Some(ApplicationCommandInteractionDataOption{
                    resolved: Some(ApplicationCommandInteractionDataOptionValue::User(user, _)),
                    ..
                }) => (user.id.0, ApplicationCommandPermissionType::User),
                Some(ApplicationCommandInteractionDataOption{
                    resolved: Some(ApplicationCommandInteractionDataOptionValue::Role(role)),
                    ..
                }) => (role.id.0, ApplicationCommandPermissionType::Role),
                _ => return Ok(())
            }
        };
        println!("{:?}", opt_who);
        let old_perms = match guild_id.get_application_command_permissions(ctx, command_id).await {
            Ok(v) => v.permissions,
            Err(_) => Vec::new()
        };
        match guild_id.create_application_command_permission(ctx, command_id, |perm| {
            old_perms.iter().for_each(|p| {
                perm.create_permission(|new_perm| new_perm
                    .id(p.id.0)
                    .kind(p.kind)
                    .permission(p.permission)
                );
            });
            perm.create_permission(|new_perm| new_perm
                .id(opt_who.0)
                .kind(opt_who.1)
                .permission(opt_type)
            );
            println!("{:?}", perm);
            perm
        }).await {
            Ok(_) => {
                let name = guild_id.name(ctx).await.unwrap_or(guild_id.to_string());
                println!("Permission for command {} setted on guild {}.", command_name, name);
            },
            Err(why) => {
                let name = guild_id.name(ctx).await.unwrap_or(guild_id.to_string());
                eprintln!("Could not set permission for command {} on guild {}: {:?}", command_name, name, why);
            }
        }

        Ok(())
    }
}