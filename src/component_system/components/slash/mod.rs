use cmp2::ApplicationCommandEmbed;
use cmp2::declarative::Node;
use opencdd_macros::commands;
use opencdd_components as cmp2;
use serenity::model::event::ReadyEvent;
use serenity::model::interactions::application_command::ApplicationCommandPermissionType;
use serenity::prelude::*;
use serenity::model::id::{UserId, ApplicationId};
use opencdd_components::message;

pub struct SlashCommand {
    app_id: ApplicationId,
    container: cmp2::container::RefContainer,
    owners: Vec<UserId>
}

impl SlashCommand {
    pub fn new(app_id: ApplicationId, container: cmp2::container::RefContainer, owners: Vec<UserId>) -> SlashCommand {
        SlashCommand {
            app_id,
            container,
            owners
        }
    }
}

#[commands]
#[group(name="slash", description="Gestion des commandes slash")]
#[group(name="permissions", description="Gérer les permissions des commandes", parent="slash")]
impl SlashCommand {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
        let container = self.container.read().await;
        let mut list_declarative = Vec::<&'static Node>::new();
        for cont in container.as_ref() {
            if let Some(node) = cont.declarative() {
                list_declarative.push(node);
            }
        }
        for guild in &ready.ready.guilds {
            let status = guild.id.set_application_commands(ctx, |v| {
                list_declarative.iter().for_each(|node| node.add_application_command(v));
                v
            }).await;
            let guild_name= guild.id.name(ctx).or_else(|| Some(guild.id.0.to_string())).unwrap();
            match status {
                Ok(_) => println!("Application commands added to {}", guild_name),
                Err(why) => {
                    println!("Error while setting application commands to \"{}\": {:?}", guild_name, why);
                }
            }
        }
    }
    #[command(name="set", description="Autoriser ou interdire une commande à un membre ou un rôle", group="permissions")]
    async fn permissions_set(
        &self,
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>, 
        #[argument(description="Le membre ou le rôle")]
        qui: cmp2::embed::Mentionable,
        #[argument(description="La commande")]
        commande: String,
        #[argument(description="Autoriser ou interdire")]
        autoriser: String
    ) {
        
    }
    #[command(name="reset", description="Retire toutes les permissions d'une commande", group="permissions")]
    async fn permissions_reset(
        &self,
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>, 
        #[argument(description="La commande")]
        commande: String
    ){

    }
    #[command(name="remove", description="Efface la permission d'un membre ou d'un rôle à une commande", group="permissions")]
    async fn permissions_remove(
        &self,
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Le membre ou le rôle")]
        qui: cmp2::embed::Mentionable,
        #[argument(description="La commande")]
        commande: String
    ){

    }
    #[command(name="list", description="Liste les permissions des commandes sur le serveur", group="permissions")]
    async fn permissions_list(
        &self,
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>
    ){
        let mut delayed = match appcmd.delayed_response(ctx).await {
            Ok(delayed) => delayed,
            Err(why) => {
                println!("Error while sending the message: {:?}", why);
                return;
            }
        };
        let guild_id = match appcmd.get_guild_id() {
            Some(guild_id) => guild_id,
            None => {
                println!("Slash permission commands can only be used in a guild");
                delayed.message = Some(message::error("Slash permission commands can only be used in a guild"));
                delayed.send(ctx, &appcmd.0).await;
                return;
            }
        };
        let mut commands = match guild_id.get_application_commands(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id);
        let perms = match guild_id.get_application_commands_permissions(ctx).await {
            Ok(v) => v,
            Err(_) => Vec::new()
        }.into_iter().filter(|c| c.application_id == self.app_id);
        let perms = perms.filter_map(|v| commands.find(|c| c.id == v.id).map(|command| (command.name, v.permissions)));

        delayed.message = {
            let mut msg = message::success("");

            if let Some(embed) = msg.last_embed_mut() {
                embed.title("Permission des commandes");
                embed.fields(perms.map(|v| {
                    let list_perm = v.1
                        .into_iter()
                        .map(|perm| {
                            let user = match perm.kind {
                                ApplicationCommandPermissionType::User => format!("<@{}>", perm.id),
                                ApplicationCommandPermissionType::Role => format!("<@&{}>", perm.id),
                                _ => "*unknown*".to_string(),
                            };
                            let permission = match perm.permission {
                                true => '✅',
                                false => '❌',
                            };
                            format!("{} {}\n", permission, user)
                        })
                        .collect::<String>();
                    (v.0.to_string(), list_perm, true)
                }));
            }
            Some(msg)
        };
        delayed.send(ctx, &appcmd.0).await;
    }

}