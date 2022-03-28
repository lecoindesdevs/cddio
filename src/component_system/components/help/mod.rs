//! Le composant help permet d'afficher une aide en fonction de la commande.
//! Il se repose sur le groupe de commande retournée par la fonction [`Component::group_parser`].

use serenity::{async_trait, utils::Colour, client::Context, model::{interactions::{application_command::ApplicationCommandInteraction, InteractionApplicationCommandCallbackDataFlags}, event::InteractionCreateEvent}, builder::CreateEmbed};

use crate::component_system::{self as cmp, command_parser::{self as cmd, Named}, manager::{ArcManager}};

use super::utils::{self, message, commands};

use super::utils::commands::*;

pub struct Help {
    manager: ArcManager,
    node: cmd::Node,
}
#[async_trait]
impl cmp::Component for Help {
    fn name(&self) -> &'static str {
        "help"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        self.r_command(fw_config, ctx, msg).await
    }

    async fn event(&self, ctx: &cmp::Context, event: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, event).await
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.node)
    }

}
#[derive(Debug, Default)]
struct HelpInfo {
    /// Nom du groupe ou de la commande
    name: String,
    /// Description du groupe ou de la commande
    desc: Option<String>,
    /// Permission (role) requise pour être utilisé
    permission: Option<String>,
    /// Si l'aide concerne un groupe, la liste des sous-groupes, s'il y en a
    groups: Option<Vec<(String, Option<String>)>>,
    /// Si l'aide concerne un groupe, la liste des sous-commande, s'il y en a
    commands: Option<Vec<(String, Option<String>)>>,
    /// Si l'aide concerne une commande, la liste des paramètres, s'il y en a
    params: Option<Vec<(String, Option<String>)>>,
    /// Si aide générale, la liste des composants
    components: Option<Vec<String>>
}

impl Help {
    pub fn new(manager: ArcManager) -> Help {
        let node = cmd::Node::new()
            .add_command(cmd::Command::new("help")
                .set_help("Affiche l'aide d'une commanded ou du bot.")
                .add_param(cmd::Argument::new("commande")
                    .set_help("Nom de la commande ou du groupe")
                )
            );
        Help { manager, node }
    }
    
    /// Helper pour le language server.
    /// rust-analyzer n'aime pas les fonctions async dans les traits
    async fn r_command(&self, _: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        let args = cmd::split_shell(&msg.content[1..]);
        let matched = match utils::try_match(ctx, msg, &self.node, args).await {
            Ok(v) => v,
            Err(e) => return e
        };
        let command = match self.commands(matched.to_command()).await {
            Ok(v) => v,
            Err(None) => return cmp::CommandMatch::NotMatched,
            Err(Some(e)) => return cmp::CommandMatch::Error(e)
        };
        
        match msg.channel_id.send_message(ctx, |m| {
            let message::Message{message, embed, .. } = command;
            m.content(message);
            m.embed(|e| {*e = embed.unwrap(); e})
        }).await {
            Ok(_) => cmp::CommandMatch::Matched,
            Err(e) => cmp::CommandMatch::Error(format!("Impossible d'envoyer le message d'aide: {}", e))
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, event: &cmp::Event) -> Result<(), String> {
        match event {
            cmp::Event::InteractionCreate(InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await,
            _ => Ok(())
        }
    }
    async fn on_applications_command(&self, ctx: &Context, app_command: &ApplicationCommandInteraction) -> Result<(), String> {
        let message::Message{message, embed, ephemeral } = match self.commands(app_command.to_command()).await {
            Ok(v) => v,
            Err(Some(e)) => return Err(e),
            Err(None) => return Ok(()),
        };
        app_command.create_interaction_response(ctx, |resp|
            resp.interaction_response_data(|data| {
                data.content(message);
                if let Some(e) = embed {
                    data.add_embed(e);
                }
                if ephemeral {
                    data.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
                }
                data
            })
        ).await
        .or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e.to_string())
        })?;
        Ok(())
    }
    fn make_help_embed(info: HelpInfo) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.color(Colour::from_rgb(0, 204, 0));
        embed.title(info.name);
        if let Some(desc) = &info.desc {
            embed.description(desc);
        }
        if let Some(permission) = &info.permission {
            embed.field("Permission", permission, true);
        }
        if let Some(groups) = &info.groups {
            let mut groups_str = String::new();
            for (name, desc) in groups {
                groups_str.push_str(&format!("**{}**", name));
                if let Some(desc) = desc {
                    groups_str.push_str(&format!(" - {}", desc));
                }
                groups_str.push_str("\n");
            }
            embed.field("Groupes", groups_str, true);
        }
        if let Some(commands) = &info.commands {
            let mut commands_str = String::new();
            for (name, desc) in commands {
                commands_str.push_str(&format!("**{}**", name));
                if let Some(desc) = desc {
                    commands_str.push_str(&format!(" - {}", desc));
                }
                commands_str.push_str("\n");
            }
            embed.field("Commandes", commands_str, true);
        }
        if let Some(params) = &info.params {
            let mut params_str = String::new();
            for (name, desc) in params {
                params_str.push_str(&format!("**{}**", name));
                if let Some(desc) = desc {
                    params_str.push_str(&format!(" - {}", desc));
                }
                params_str.push_str("\n");
            }
            embed.field("Paramètres", params_str, true);
        }
        if let Some(components) = &info.components {
            let mut components_str = String::new();
            for name in components {
                components_str.push_str(&format!("**{}**\n", name));
            }
            embed.field("Composants", components_str, true);
        }
        embed
    }
    async fn commands(&self, command: commands::Command) -> Result<message::Message, Option<String>> {
        match command.fullname().as_str() {
            "help" => {
                let words = match command.get_argument("commande") {
                    Some(Argument{value: Value::String(v), ..}) => Some(v),
                    Some(_) => return Err(Some("Argument `commande` invalide".to_string())),
                    None => None
                };
                let help_info = match words {
                    Some(words) => self.help_components(words).await.or_else(|_| Err("Aucune aide trouvé.".to_string())),
                    None => self.list_commands().await
                };
                
                let (msg_to_send, ephemeral) = match help_info {
                    Ok(v) => {
                        (Self::make_help_embed(v), false)
                    },
                    Err(e) => {
                        let mut embed = CreateEmbed::default();
                        embed.color(Colour::from_rgb(204, 0, 0));
                        embed.title("Erreur");
                        embed.description(e);
                        (embed, true)
                    }
                };
                Ok(message::Message {
                    message: String::new(),
                    embed: Some(msg_to_send),
                    ephemeral,
                    ..Default::default()
                })
            }
            _ => Err(None)
        }
    }
    async fn help_components<'a, 'b>(&'a self, command_name: &'b str) -> Result<HelpInfo, ()> {
        let comps = self.manager.read().await;
        let comps = comps.get_components();
        
        for cmp in comps {
            let node = match cmp.node() {
                Some(v) => v,
                None => continue
            };
            let (_, cmd) = match node.list_commands().into_iter().find(|(cmd_name, _)| cmd_name == command_name) {
                Some(v) => v,
                None => continue
            };
            return Self::help_command(cmd, command_name);
        }
        Err(())
    }
    fn help_command<'a, 'b>(command: &'a cmd::Command, command_name: &'b str) -> Result<HelpInfo, ()> {
        let params = command.params.iter()
            .map(|param| (format!("{} <{}>", param.name(), param.value_type_str()), param.help.clone()))
            .collect::<Vec<(String, Option<String>)>>();
        Ok(HelpInfo{
            name: format!("{} (Commande)", command_name),
            permission: None,
            desc: command.help().and_then(|v| Some(v.to_string())),
            params: if params.is_empty() {None} else {Some(params)},
            .. Default::default()
        })
    }
    
    async fn list_commands(&self) -> Result<HelpInfo, String> {
        let comps = self.manager.read().await;
        let comps = comps.get_components();
        let mut commands = Vec::new();
        for comp in comps {
            if let Some(node) = comp.node() {
                commands.extend(
                    node.list_commands()
                        .into_iter()
                        .map(|(name, cmd)| (name, cmd.help().and_then(|help| Some(help.to_string()))))
                );
            }
        }
        Ok(HelpInfo{
            name: "Liste des commandes".to_string(),
            commands: Some(commands),
            .. Default::default()
        })
    }
   
}