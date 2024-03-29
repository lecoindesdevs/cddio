use chrono::{DateTime, Utc};
use crate::{log_error, log_warn};
use cddio_core::{message, ApplicationCommandEmbed};
use serenity::{
    client::Context,
    model::id::*, 
    async_trait
};
use serde::{Deserialize, Serialize};
use super::utils;
pub const ROLE_MUTED: &str = "muted";

use super::task;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sanction {
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub user_by: UserId,
    pub data: SanctionType,
}
#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SanctionType {
    Ban{
        #[serde_as(as = "Option<serde_with::TimestampSeconds>")]
        until: Option<DateTime<Utc>>,
        historique: u8,
        reason: String,
    },
    Mute {
        #[serde_as(as = "Option<serde_with::TimestampSeconds>")]
        until: Option<DateTime<Utc>>,
        reason: String,
    },
    Kick {
        reason: String,
    },
    Unban,
    Unmute,
}

impl Sanction {
    pub fn from_app_command(app_cmd: &ApplicationCommandEmbed, member: UserId, data: SanctionType) -> Self {
        Self {
            user_id: member,
            guild_id: app_cmd.get_guild_id().unwrap(),
            user_by: app_cmd.0.user.id,
            data,
        }
    }
    pub const fn name(&self) -> &'static str {
        match &self.data {
            SanctionType::Ban{..} => "Ban",
            SanctionType::Mute{..} => "Mute",
            SanctionType::Kick{..} => "Kick",
            SanctionType::Unban{..} => "Unban",
            SanctionType::Unmute{..} => "Unmute",
        }
    }
    pub const fn preterite(&self) -> &'static str {
        match &self.data {
            SanctionType::Ban{..} => "banni",
            SanctionType::Mute{..} => "mute",
            SanctionType::Kick{..} => "kick",
            SanctionType::Unban{..} => "débanni",
            SanctionType::Unmute{..} => "démute",
        }
    }
    pub async fn apply(&self, ctx: &Context) -> serenity::Result<()> {
        let (guild_id, user_id) = (self.guild_id, self.user_id);
        match &self.data {
            SanctionType::Ban{historique, reason, ..} => guild_id.ban_with_reason(ctx, user_id, *historique, reason).await,
            SanctionType::Mute{ until, ..} => {
                use serenity::model::timestamp::Timestamp;
                let timestamp = match until {
                    Some(until) => until.timestamp(),
                    None => {
                        Utc::now()
                            .checked_add_signed(chrono::Duration::minutes(10))
                            .ok_or_else(|| serenity::Error::Other("Impossible de trouver le temps"))?
                            .timestamp()
                    }
                };
                let timestamp = Timestamp::from_unix_timestamp(timestamp).unwrap();
                guild_id.member(ctx, user_id).await?.disable_communication_until_datetime(ctx, timestamp).await
            },
            SanctionType::Kick{reason} => guild_id.kick_with_reason(ctx, user_id, reason).await,
            SanctionType::Unban => guild_id.unban(ctx, user_id).await,
            SanctionType::Unmute => guild_id.member(ctx, user_id).await?.enable_communication(ctx).await,
        }
    }
    pub fn user_id(&self) -> UserId {
        self.user_id
    }
    pub fn guild_id(&self) -> GuildId {
        self.guild_id
    }
    pub fn data(&self) -> &SanctionType {
        &self.data
    }
    #[inline]
    pub async fn to_user_message(&self, ctx: &Context) -> message::Message {
        let guild_id = self.guild_id();
        let guild_name = guild_id
            .to_guild_cached(ctx)
            .map(|v| v.name)
            .or_else(|| guild_id
                .to_guild_cached(ctx)
                .map(|v| v.name)
            )
            .unwrap_or_else(|| guild_id.to_string());
        
        self.to_message(message::COLOR_INFO, format!("Vous avez été {} du serveur {}", self.preterite(), guild_name))
    }
    #[inline]
    pub async fn to_server_message(&self, ctx: &Context) -> message::Message {
        let user = self.user_id().to_user(ctx).await.unwrap();
        self.to_message(message::COLOR_SUCCESS, format!("{} a été {}", user.name, self.preterite()))
    }
    fn estimation_time(date: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let diff = date.signed_duration_since(now);
        if (diff.num_days()+15)/30 > 0 {
            format!("{} mois", (diff.num_days()+15)/30)
        } else if diff.num_days() > 0 {
            format!("{} jours", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{} heures", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{} minutes", diff.num_minutes())
        } else {
            format!("{} secondes", diff.num_seconds())
        }
    }
    fn format_date(date: &DateTime<Utc>) -> String {
        format!("{} (environ {})", date.format("%d %B %Y à %H:%M:%S"), Sanction::estimation_time(date))
    }
    
    fn to_message<S: ToString>(&self, color: serenity::utils::Colour, description: S) -> message::Message {
        let mut m = message::Message::new();
        m.add_embed(|e| {
            e.title(self.name())
                .description(description)
                .color(color);
            if let SanctionType::Ban{until: Some(until), ..} | SanctionType::Mute{until: Some(until), ..} = &self.data {
                e.field("Temps", Self::format_date(until), true);
            }
            if let SanctionType::Ban{reason, ..} | SanctionType::Mute{reason, ..} | SanctionType::Kick{reason, ..} = &self.data {
                e.field("Raison", reason, true);
            }
            e
        });
        m
    }
    async fn username(ctx: &Context, user_id: UserId) -> String {
        user_id.to_user(ctx).await.map(|user| utils::user_fullname(&user)).unwrap_or_else(|_| user_id.to_string())
    }
    pub async fn to_log(&self, ctx: &Context, user_by: UserId) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut log = String::new();
        let now = chrono::Local::now();
        let user_by = Self::username(ctx, user_by).await;
        write!(log, "{:=<10}\n", "")?;
        write!(log, "When: {}\n", now.format("%d/%m/%Y %H:%M:%S"))?;
        write!(log, "By: {}\n", user_by)?;
        let user_id = self.user_id();
        match &self.data {
            SanctionType::Ban{ until, reason, ..} => {
                let user = Self::username(ctx, user_id).await;
                write!(log, "What: {}\n", "Ban")?;
                write!(log, "Who: {}\n", user)?;
                write!(log, "Why: {}\n", reason)?;
                if let Some(until) = until {
                    write!(log, "Until: {}\n", until)?;
                }
            },
            SanctionType::Mute{ until, reason, ..} => {
                let user = Self::username(ctx, user_id).await;
                write!(log, "What: {}\n", "Mute")?;
                write!(log, "Who: {}\n", user)?;
                write!(log, "Why: {}\n", reason)?;
                if let Some(until) = until {
                    write!(log, "Until: {}\n", until)?;
                }
            },
            SanctionType::Kick{ reason, ..} => {
                let user = Self::username(ctx, user_id).await;
                write!(log, "What: {}\n", "Kick")?;
                write!(log, "Who: {}\n", user)?;
                write!(log, "Why: {}\n", reason)?;
            },
            SanctionType::Unban => {
                let user = Self::username(ctx, user_id).await;
                write!(log, "What: {}\n", "Unban")?;
                write!(log, "Who: {}\n", user)?;
            },
            SanctionType::Unmute => {
                let user = Self::username(ctx, user_id).await;
                write!(log, "What: {}\n", "Unmute")?;
                write!(log, "Who: {}\n", user)?;
            }
        }
        Ok(log)
    }
    async fn undo(&self, ctx: &Context) {
        let result = match self.data {
            SanctionType::Ban{..} => Sanction{data: SanctionType::Unban, ..*self}.apply(ctx).await,
            SanctionType::Mute{..} => Sanction{data: SanctionType::Unmute, ..*self}.apply(ctx).await,
            _ => Err(serenity::Error::Other("Sanction impossible à annuler."))
        };
        if let Err(e) = result {
            log_error!("Impossible de rétablir la sanction {}: {}", self.user_id(), e);
        }
    }
}
#[async_trait]
impl task::DataFunc for Sanction {
    type Persistent = Context;
    async fn run(&self, ctx: &Context) -> Result<(), String> {
        Ok(self.undo(ctx).await)
    }
}