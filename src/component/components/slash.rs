use serenity::{async_trait, builder::CreateApplicationCommands};
use crate::component::{self as cmp, command_parser::{self as cmd, Named}, manager::{ArcManager}};

use crate::component::slash;
pub struct SlashInit {
    manager: ArcManager
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
}

impl SlashInit {
    pub fn new(manager: ArcManager) -> Self {
        SlashInit {
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

                for guild in guilds {
                    match guild.id().set_application_commands(ctx, |v| {
                        *v = app_commands.clone();
                        v
                    }).await {
                        Ok(_) => (),
                        Err(why) => {
                            let name = guild.id().name(ctx).await.unwrap_or(guild.id().to_string());
                            eprintln!("Could not set application commands for guild {}: {:?}", name, why);
                        }
                    }
                }
                println!("Slash commands setted.");
            },
            // cmp::Event::InteractionCreate(e) => e.interaction.application_command().unwrap().
            _ => (),
        }
        Ok(())
    }
}