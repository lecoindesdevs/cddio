mod time;

use crate::component::{self as cmp, command_parser as cmd};
use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, model::{interactions::application_command::ApplicationCommandInteraction, id::{ApplicationId, GuildId, UserId}, guild::Guild}, client::Context};

use super::utils::{app_command::{ApplicationCommandEmbed, get_argument}, Data};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct ModerationData {
    banned_until: Vec<(GuildId, UserId, i64)>,
    mute_until: Vec<(GuildId, UserId, i64)>,
}
#[derive(Clone, Debug)]
struct Moderation {
    node: cmd::Node,
    app_id: ApplicationId,
    data: RwLock<Data<ModerationData>>,
}

#[async_trait]
impl cmp::Component for Moderation {
    fn name(&self) -> &'static str {
        "mod"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        cmp::CommandMatch::NotMatched
    }

    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.node)
    }
}

impl Moderation {
    fn new(app_id: ApplicationId) -> Moderation {
        let ban = cmd::Command::new("ban")
            .set_help("Bannir un membre. Temporaire si l'argument for est présent.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à bannir")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("raison")
                .set_value_type(cmd::ValueType::String)
                .set_help("La raison du ban")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pendant")
                .set_value_type(cmd::ValueType::String)
                .set_help("Pendant combien de temps")
            );
        let mute = cmd::Command::new("mute")
            .set_help("Mute un membre. Temporaire si l'argument for est présent.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à bannir")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("raison")
                .set_value_type(cmd::ValueType::String)
                .set_help("La raison du mute")
            )
            .add_param(cmd::Argument::new("pendant")
                .set_value_type(cmd::ValueType::String)
                .set_help("Pendant combien de temps")
            );
        let node = cmd::Node::new()
            .add_command(ban)
            .add_command(mute);
        Moderation {
            node: cmd::Node::new(),
            app_id,
            data: match Data::from_file_default("moderation") {
                Ok(data) => RwLock::new(data),
                Err(e) => panic!("Data tickets: {:?}", e)
            }
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        use serenity::model::event::InteractionCreateEvent;
        use serenity::model::interactions::Interaction::*;
        match evt {
            cmp::Event::InteractionCreate(InteractionCreateEvent{interaction: ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await,
            _ => Ok(()),
        }
    }
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
            "ban" => self.ban(ctx, guild_id, &app_cmd).await,
            // "mute" => self.mute(ctx, guild_id, app_cmd).await,
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
    async fn ban<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: &ApplicationCommandEmbed<'a>) -> Result<(), String> {
        let user = match get_argument!(app_cmd, "qui", User) {
            Some(v) => v.0,
            None => return Err("Vous devez mentionner un membre.".into())
        };
        let reason = match get_argument!(app_cmd, "raison", String) {
            Some(v) => v,
            None => return Err("La raison d'un bannissement est nécessaire.".into())
        };
        let time = match get_argument!(app_cmd, "pendant", String) {
            Some(v) => Some(time::parse(v)?),
            None => None
        };
        if user.id == app_cmd.0.member.as_ref().unwrap().user.id {
            return Err("Vous ne pouvez pas vous bannir.".into());
        }
        let member = guild_id.member(ctx, user.id).await.or_else(|e| {
            eprintln!("Cannot get member: {}", e);
            Err(e.to_string())
        })?;
        // TODO: Envoyer un message d'avertissement au membre banni
        if let Err(e) = member.ban_with_reason(ctx, 0, reason).await {
            return Err(format!("Impossible de bannir le membre: {}", e));
        }
        if let Some(time) = time {
            let time = chrono::Duration::seconds(time as _);
            let when = chrono::Utc::now() + time;
            let when = when.timestamp();
            let mut data = self.data.write().await;
            let mut data = data.write();
            match data.banned_until.iter_mut().find(|(gid, uid, _)| uid == &user.id && gid == &guild_id) {
                Some((_, _, t)) => {
                    *t = when;
                },
                None => data.banned_until.push((guild_id, user.id, when))
            }
            // TODO: spawn un thread qui va débannir le membre après le temps
        }
        Ok(())
    }
}