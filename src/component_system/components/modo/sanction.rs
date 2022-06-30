use chrono::{DateTime, Utc};
use log::*;
use opencdd_components::message;
use serenity::{
    client::Context,
    model::id::*, 
    async_trait
};
use serde::{Deserialize, Serialize};
use super::utils;
const ROLE_MUTED: &str = "muted";

use crate::component_system::components::utils::task;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Sanction {
    pub user_id: UserId,
    pub guild_id: GuildId,
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
            SanctionType::Ban{historique, reason, ..} => {
                guild_id.ban_with_reason(ctx, user_id, *historique, reason).await
            },
            SanctionType::Mute{..} => {
                let role = guild_id
                    .roles(ctx).await?
                    .into_iter()
                    .find(|(_, r)| r.name == ROLE_MUTED)
                    .map(|(id, _)| id);
                match role {
                    Some(role) => {
                        let mut member = guild_id.member(ctx, user_id).await?;
                        member.add_role(ctx, role).await?;
                        Ok(())
                    },
                    None => {
                        warn!("Impossible de trouver le rôle \"{}\" dans le serveur {}", ROLE_MUTED, guild_id);
                        Err(serenity::Error::Other("Impossible de trouver le rôle \"muted\" dans le serveur"))
                    }
                } 
            },
            SanctionType::Kick{reason} => {
                guild_id.kick_with_reason(ctx, user_id, reason).await
            },
            SanctionType::Unban => {
                guild_id.unban(ctx, user_id).await
            },
            SanctionType::Unmute => {
                let role = guild_id
                    .roles(ctx).await?
                    .into_iter()
                    .find(|(_, r)| r.name == ROLE_MUTED)
                    .map(|(id, _)| id);
                match role {
                    Some(role) => {
                        let mut member = guild_id.member(ctx, user_id).await?;
                        member.remove_role(ctx, role).await?;
                        Ok(())
                    },
                    None => {
                        warn!("Impossible de trouver le rôle \"muted\" dans le serveur {}", guild_id);
                        Err(serenity::Error::Other("Impossible de trouver le rôle \"muted\" dans le serveur"))
                    }
                } 
            }
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
        format!("{} (durée: {})", date.format("%d %B %Y à %H:%M:%S"), Sanction::estimation_time(date))
    }
    
    fn to_message<S: ToString>(&self, color: serenity::utils::Colour, description: S) -> message::Message {
        let mut m = message::Message::new();
        m.add_embed(|e| {
            e.title(self.name());
            e.description(description);
            e.color(color);
            match &self.data {
                SanctionType::Ban{until, reason, ..} => {
                    if let Some(until) = until {
                        e.field("Temps", Self::format_date(&until), true);
                    }
                    e.field("Raison", reason, true);
                },
                SanctionType::Mute{until, reason, ..} => {
                    if let Some(until) = until {
                        e.field("Temps", Self::format_date(&until), true);
                    }
                    e.field("Raison", reason, true);
                },
                SanctionType::Kick{reason, ..} => {
                    e.field("Raison", reason, true);
                },
                _ => ()
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
            error!("Impossible de rétablir la sanction {}: {}", self.user_id(), e);
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