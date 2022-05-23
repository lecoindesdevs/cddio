use cmp2::declarative::Node;
use opencdd_macros::commands;
use opencdd_components as cmp2;
use serenity::model::event::ReadyEvent;
use serenity::prelude::*;
use serenity::model::id::UserId;

pub struct SlashCommand {
    container: cmp2::ComponentContainer,
    owners: Vec<UserId>
}

impl SlashCommand {
    pub fn new(container: cmp2::ComponentContainer, owners: Vec<UserId>) -> SlashCommand {
        SlashCommand {
            container,
            owners
        }
    }
}

#[commands]
impl SlashCommand {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
        let container = self.container.as_ref();
        let mut list_declarative = Vec::<&'static Node>::new();
        for cont_rc in container.iter() {
            let cont = cont_rc.lock().await;
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
}