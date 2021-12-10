mod time;

use crate::component::{self as cmp, command_parser as cmd};
use chrono::{DateTime, Utc};
use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, model::{interactions::application_command::ApplicationCommandInteraction, id::{ApplicationId, GuildId, UserId}, guild::{Guild, Member}, event::ReadyEvent}, client::Context};

use super::utils::{app_command::{ApplicationCommandEmbed, get_argument}, Data, message};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct ModerationData {
    banned_until: Vec<(u64, i64)>, // (user_id, time)
    mute_until: Vec<(u64, i64)>, // (user_id, time)
}
#[derive(Clone, Debug)]
pub struct Moderation {
    node: cmd::Node,
    app_id: ApplicationId,
    data: RwLock<Data<ModerationData>>,
}

#[async_trait]
impl cmp::Component for Moderation {
    fn name(&self) -> &'static str {
        "mod"
    }

    async fn command(&self, _: &cmp::FrameworkConfig, _: &cmp::Context, _: &cmp::Message) -> cmp::CommandMatch {
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
    pub fn new(app_id: ApplicationId) -> Moderation {
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
            node,
            app_id,
            data: match Data::from_file_default("moderation") {
                Ok(data) => RwLock::new(data),
                Err(e) => panic!("Data moderation: {:?}", e)
            }
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        use serenity::model::event::InteractionCreateEvent;
        use serenity::model::interactions::Interaction::*;
        use cmp::Event::*;
        match evt {
            Ready(ReadyEvent{ready, ..}) => self.on_ready(ctx, ready).await,
            InteractionCreate(InteractionCreateEvent{interaction: ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await,
            _ => Ok(()),
        }
    }
    async fn on_ready(&self, ctx: &cmp::Context, ready: &serenity::model::gateway::Ready) -> Result<(), String> {
        let (banned_until, guild_id)= {
            let data = self.data.read().await;
            let data = data.read();
            
            let guild_ids = ready.guilds.iter().map(|g| g.id()).collect::<Vec<_>>();
            let guild_id = guild_ids.first().cloned().ok_or("No guild found".to_string())?;
            (data.banned_until.clone(), guild_id)
        };

        banned_until.iter().cloned().for_each(|(user_id, time)| {
            let ctx = ctx.clone();
            let data = self.data.clone();
            tokio::spawn(async move {
                let date_time = DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(time, 0), Utc);
                let member = match guild_id.member(&ctx, user_id).await {
                    Ok(member) => member,
                    Err(e) => {
                        eprintln!("tempban execution: Error getting member {}: {}", user_id, e);
                        return;
                    },
                };
                Self::unban_thread(ctx, member, date_time, data);
            });
        });
        Ok(())
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
            "ban" => self.ban(ctx, guild_id, &app_cmd).await?,
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
    fn unban_thread(ctx: Context, member: Member, date_time: DateTime<Utc>, data: RwLock<Data<ModerationData>>) {
        tokio::spawn(async move {
            let duration = date_time - chrono::Utc::now();
            if duration.num_seconds()>0 {
                tokio::time::sleep(duration.to_std().unwrap()).await;
            }
            if match member.unban(ctx).await {
                Ok(_) => true,
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg == "Unknown Ban" {
                        true
                    } else {
                        eprintln!("Error unbanning {}: {}", member.user.id, err_msg);
                        false
                    }
                }
            } {
                let mut data = data.write().await;
                let mut data = data.write();
                let banned_until = &mut data.banned_until;
                let idx = banned_until.iter().position(|(user_id, _)| user_id == &member.user.id.0).unwrap();
                data.banned_until.remove(idx);
                println!("Membre {} débanni", member.user.name)
                
            }
        });
    }
    async fn ban<'a>(&self, ctx: &Context, guild_id: GuildId, app_cmd: &ApplicationCommandEmbed<'a>) -> Result<message::Message, String> {
        let user = match get_argument!(app_cmd, "qui", User) {
            Some(v) => v.0,
            None => return Err("Vous devez mentionner un membre.".into())
        };
        let reason = match get_argument!(app_cmd, "raison", String) {
            Some(v) => v,
            None => return Err("La raison est nécessaire.".into())
        };
        let time = match get_argument!(app_cmd, "pendant", String) {
            Some(v) => {
                let duration_second = time::parse(v)? as _;
                let duration = chrono::Duration::seconds(duration_second);
                let time_point = chrono::Utc::now() + duration;
                Some((time_point.timestamp(), time_point))
            },
            None => None
        };
        if user.id == app_cmd.0.member.as_ref().unwrap().user.id {
            return Err("Vous vous êtes mentionné vous meême dans `qui`.".into());
        }
        let username = format!("{}#{}", user.name, user.discriminator);
        let guild_name = match guild_id.name(ctx).await {
            Some(v) => v,
            _ => "Coin des développeurs".to_string()
        };
        let member = guild_id.member(ctx, user.id).await.or_else(|e| {
            eprintln!("Impoossible d'obtenir le membre depuis le serveur: {}", e);
            Err(e.to_string())
        })?;
        match user.direct_message(ctx, |msg| {
            msg.embed(|embed| {
                embed.title("Vous avez été banni du serveur.");
                if let Some((_, date_time)) = time {
                    embed.description(format!("Vous avez été banni temporairement du serveur {}, le bannissement prendra fin le {} UTC.", guild_name, date_time.format("%d/%m/%Y à %H:%M:%S")));
                } else {
                    embed.description("Vous avez été banni du serveur.");
                }
                embed.field("Raison", reason, false);
                
                embed.color(0xFF0000);
                embed
            });
            msg
        }).await {
            Ok(_) => (),
            Err(e) => eprintln!("Impossible d'envoyer le message de bannissement à l'utilisateur {}: {}", username, e)
        };

        if let Err(e) = member.ban_with_reason(ctx, 0, reason).await {
            return Err(format!("Impossible de bannir le membre: {}", e));
        }
        let msg = match time {
            Some((timestamp, date_time)) => {
                let mut data = self.data.write().await;
                let mut data = data.write();
                match data.banned_until.iter_mut().find(|(uid, _)| uid == &user.id.0) {
                    Some((_, t)) => *t = timestamp,
                    None => data.banned_until.push((user.id.0, timestamp))
                };
                let formatted_date = date_time.format("%d/%m/%Y à %H:%M:%S");
                Self::unban_thread(ctx.clone(), member, date_time, self.data.clone());
                format!("Le membre <@{}> ({}) a été banni temporairement du serveur {}, fini le {} UTC.", user.id, username, guild_name, formatted_date)
            },
            None => format!("Le membre <@{}> ({}) a été banni du serveur {}.", user.id, username, guild_name)
        };
        println!("{}", msg);
        let mut msg = message::success(msg);
        msg.embed.as_mut().unwrap().field("Raison", reason, false);
        Ok(msg)
    }
}