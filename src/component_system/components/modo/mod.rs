

use std::collections::HashMap;
use chrono::{Duration, DateTime, Utc};
use log::*;

use async_std::io::WriteExt;
use futures_locks::RwLock;
use opencdd_components::{ApplicationCommandEmbed, message};
use opencdd_macros::commands;
use serenity::{
    client::Context,
    model::{
        id::*,
        event::*
    }, async_trait
};
use serde::{Deserialize, Serialize};

use super::utils;
use super::utils::time_parser as time;

use crate::component_system::components::utils::task;

const ROLE_MUTED: &str = "muted";

struct RegistryFile {
    path_file: std::path::PathBuf,
    tasks: RwLock<HashMap<task::TaskID, task::Task<Sanction>>>,
    task_counter: RwLock<task::TaskID>
}

impl RegistryFile {
    async fn from_file(path_file: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let res = Self {
            path_file: path_file.as_ref().to_path_buf(),
            tasks: RwLock::new(HashMap::new()),
            task_counter: RwLock::new(1)
        };
        res.load().await?;
        Ok(res)
    }
    async fn save(&self) -> Result<(), String> {
        let log_error = |msg, e| {
            let e = format!("modo::RegistryFile::save: {}: {}", msg, e);
            error!("{}", e);
            e
        };
        let mut file = async_std::fs::File::create(&self.path_file).await
            .map_err(|e| log_error(format!("Unable to open/create file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;

        let tasks = self.tasks.read().await;
        let data = ron::to_string(&*tasks)
            .map_err(|e| log_error(format!("Unable to serialize tasks"), e.to_string()))?;
        file.write_all(data.as_bytes()).await
            .map_err(|e| log_error(format!("Unable to open/create file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;
        Ok(())
    }
    async fn load(&self) -> Result<(), String> {
        let log_error = |msg, e| {
            let e = format!("modo::RegistryFile::load: {}: {}", msg, e);
            error!("{}", e);
            e
        };
        if self.path_file.exists() {
            let data = std::fs::read_to_string(&self.path_file)
                .map_err(|e| log_error(format!("Unable to read file at '{}'", self.path_file.to_string_lossy()), e.to_string()))?;
            let tasks: HashMap<_,_> = ron::from_str(&data)
                .map_err(|e| log_error(format!("Unable to parse tasks"), e.to_string()))?;
            let highest_id = tasks.iter().map(|(id, _)| *id).max().unwrap_or(0);
            *self.tasks.write().await = tasks;
            *self.task_counter.write().await = highest_id + 1;
        }
        Ok(())
    }
}
#[async_trait]
impl task::Registry for RegistryFile {
    type Data = Sanction;
    async fn register(&mut self, task: task::Task<Self::Data>) -> Result<task::TaskID, String> {
        let id = self.task_counter.read().await.clone();
        self.tasks.write().await.insert(id, task);
        *self.task_counter.write().await += 1;
        match self.save().await {
            Ok(_) => Ok(id),
            Err(e) => Err(e)
        }
    }
    async fn unregister(&mut self, id: task::TaskID) -> Result<(), String> {
        self.tasks.write().await.remove(&id);
        match self.save().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    async fn get(&self, id: task::TaskID) -> Option<task::Task<Self::Data>> {
        self.tasks.read().await.iter().find(|(vid, _)| vid == &&id).map(|(_, task)| task.clone())
    }

    async fn get_all(&self) -> Vec<(task::TaskID, task::Task<Self::Data>)> {
        self.tasks.read().await.iter().map(|v| (*v.0, v.1.clone())).collect()
    }
}

pub struct Moderation {
    tasks: RwLock<Option<task::TaskManager<Sanction, RegistryFile, Context>>>,
}
impl Moderation {
    pub fn new() -> Self {
        Moderation {
            tasks: RwLock::new(None)
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Sanction {
    user_id: UserId,
    guild_id: GuildId,
    data: SanctionType,
}
#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
enum SanctionType {
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
        match self.data {
            SanctionType::Ban{..} => {
                let username = self.user_id.to_user(&ctx).await
                    .and_then(|v| Ok(v.name))
                    .or_else::<(), _>(|_| Ok(self.user_id.to_string()))
                    .unwrap();
                match self.guild_id.unban(&ctx, self.user_id).await {
                    Ok(_) => println!("Membre \"{}\" débanni", username),
                    Err(e) => println!("Erreur lors du déban de {} : {}", username, e)
                }
            },
            SanctionType::Mute{..} => {
                let mut member = match self.guild_id.member(&ctx, self.user_id).await{
                    Ok(m) => m,
                    Err(e) => {
                        println!("Impossible de trouver le membre {} dans le serveur {}: {}", self.user_id, self.guild_id, e.to_string());
                        return
                    }
                };
                let roles = match self.guild_id.roles(ctx).await{
                    Ok(r) => r,
                    Err(e) => {
                        println!("Impossible de trouver les rôles du serveur {}: {}", self.guild_id, e.to_string());
                        return
                    }
                };
                let role_muted = match roles.into_iter().find(|(_, r)| r.name.as_str() == ROLE_MUTED) {
                    Some(r) => r,
                    None => {
                        println!("Impossible de trouver le rôle '{}' dans le serveur {}", ROLE_MUTED, self.guild_id);
                        return
                    }
                };
                match member.remove_role(&ctx, role_muted.0).await {
                    Ok(_) => println!("Membre {} débanni", member.display_name()),
                    Err(e) => println!("Impossible de retirer le rôle {} (id: {}) du membre \"{}\" dans le serveur {}: {}", role_muted.1.name, role_muted.0, member.display_name(), self.guild_id, e.to_string())
                };
            },
            _ => ()
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

#[commands]
impl Moderation {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, _: &ReadyEvent) {
        let mut tasks = self.tasks.write().await;
        let ctx = ctx.clone();
        match &mut *tasks {
            Some(tasks) => tasks.reset_persistent(ctx.clone()),
            None => {
                let registry = RegistryFile::from_file("./data/moderation2.ron").await.unwrap();
                let new_tasks = task::TaskManager::new(registry, ctx.clone());
                *tasks = Some(new_tasks);
            }
        }
    }
    #[event(GuildBanAdd)]
    async fn on_ban_add(&self, ctx: &Context, event: &GuildBanAddEvent) {
        // let guild_id = event.guild_id;
        // let user_by = event.user;

    }
    #[command(description="Banni un membre du serveur")]
    pub async fn ban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à bannir", name="qui")]
        member: UserId,
        #[argument(description="Raison du ban")]
        raison: String,
        #[argument(description="Supprimer l'historique du membre (nombre de jours de 0 à 7)", name="historique")]
        del_msg: Option<u8>,
        // #[argument(description="Durée du ban")]
        // duration: Option<String>
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        // let duration = match duration.map(|v| time::parse(v)) {
        //     Some(Ok(v)) => Some(Duration::seconds(v as _)),
        //     Some(Err(e)) => {
        //         match app_cmd.direct_response(ctx, message::error(format!("Impossible de parser la durée: {}", e))).await {
        //             Ok(_) => (),
        //             Err(e) => error!("Impossible de renvoyer une réponse directe: {}", e)
        //         }
        //         return;
        //     }
        //     None => None
        // };
        let until = None;
        
        self.do_sanction(ctx, app_cmd, Sanction {
            user_id: member,
            guild_id,
            data : SanctionType::Ban{
                reason: raison,
                until,
                historique: del_msg.map(|v| v.clamp(0, 7)).unwrap_or(0)
            }
        }).await;
        
    }
    #[command(description="Expulse un membre du serveur")]
    pub async fn kick(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à expulser", name="qui")]
        member: UserId,
        #[argument(description="Raison de l'expulsion")]
        reason: String
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        self.do_sanction(ctx, app_cmd, Sanction {
            user_id: member,
            guild_id,
                data: SanctionType::Kick{
                reason
            }
        }).await;
    }
    #[command(description="Mute un membre du serveur")]
    pub async fn mute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à mute", name="qui")]
        member: UserId,
        #[argument(description="Raison du ban")]
        raison: String,
        #[argument(description="Durée du mute")]
        duration: Option<String>
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        let until = match duration.map(|v| time::parse(v)) {
            Some(Ok(v)) => {
                Some(Utc::now() + Duration::seconds(v as _))
            },
            Some(Err(e)) => {
                match app_cmd.direct_response(ctx, message::error(format!("Impossible de parser la durée: {}", e))).await {
                    Ok(_) => (),
                    Err(e) => error!("Impossible de renvoyer une réponse directe: {}", e)
                }
                return;
            }
            None => None
        };
        // let duration = None;
        
        self.do_sanction(ctx, app_cmd, Sanction {
            user_id: member,
            guild_id,
            data: SanctionType::Mute{
                reason: raison,
                until,
            }
        }).await;
    }
    #[command(description="Débanni un membre du serveur")]
    pub async fn unban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à débannir", name="qui")]
        member: UserId
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        self.do_sanction(ctx, app_cmd, Sanction {
            user_id: member,
            guild_id,
            data: SanctionType::Unban
        }).await;
    }
    #[command(description="Démute un membre du serveur")]
    pub async fn unmute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à démute", name="qui")]
        member: UserId
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        self.do_sanction(ctx, app_cmd, Sanction{
            user_id: member,
            guild_id, 
            data: SanctionType::Unmute
        }).await;
    }

}

impl Moderation {
    async fn do_sanction(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>, sanction: Sanction) {
        match app_cmd.get_guild_id() {
            None => {
                match app_cmd.direct_response(ctx, message::error("Vous devez être dans un serveur pour utiliser cette commande")).await  {
                    Ok(_) => (),
                    Err(e) => error!("Impossible de renvoyer une réponse directe: {}", e)
                }
                return;
            }
            _ => (),
        };
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(v) => v,
            Err(e) => {
                error!("Impossible de créer une réponse diférée: {}", e.to_string()); 
                return;
            }
        };
        let user_id = sanction.user_id();
        match &sanction.data {
            SanctionType::Ban { .. } | SanctionType::Mute { .. } | SanctionType::Kick { .. } => {
                let user = match user_id.to_user(&ctx).await {
                    Ok(v) => Some(v),
                    Err(_) => None
                };
                if let Some(user) = &user {
                    let msg = sanction.to_user_message(&ctx).await;
                    let res = user.direct_message(ctx, |create_msg| {
                        *create_msg = msg.into();
                        create_msg
                   }).await;
                   match res {
                        Ok(_) => (),
                        Err(e) => warn!("L'utilisateur {} a été trouvé mais impossible de lui envoyer un message: {}", user_id, e.to_string())
                   }
                }
            }
            _ => ()
        }
        match sanction.apply(ctx).await {
            Ok(_) => (),
            Err(e) => {
                let msg = message::error(format!("Impossible d'appliquer la sanction: {}", e.to_string()));
                match resp.send_message(msg).await {
                    Ok(_) => (),
                    Err(e) => warn!("Impossible de renvoyer la réponse d'une commande: {}", e.to_string())
                }
                return;
            }
        };
        let msg = sanction.to_server_message(ctx).await;
        match sanction {
            Sanction { data: SanctionType::Ban { until: Some(until), .. } | SanctionType::Mute { until: Some(until), .. }, .. } => {
                let mut tasks = self.tasks.write().await;
                let tasks = tasks.as_mut().unwrap();
                match tasks.add(sanction, until).await {
                    Ok(_) => (),
                    Err(e) => {
                        let msg = message::error(format!("Impossible d'ajouter la sanction à la liste: {}", e.to_string()));
                        match resp.send_message(msg).await {
                            Ok(_) => (),
                            Err(e) => warn!("Impossible de renvoyer la réponse d'une commande: {}", e.to_string())
                        }
                        return;
                    }
                }
            },
            _ => ()
        }
        
        match resp.send_message(msg).await{
            Ok(_) => (),
            Err(e) => warn!("Impossible de renvoyer la réponse d'une commande: {}", e.to_string())
        }
    }
}