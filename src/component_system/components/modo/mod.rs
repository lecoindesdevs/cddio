

mod time;
use chrono::Duration;
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

use crate::component_system::components::utils::task;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum TypeModeration {
    Ban,
    Mute,
    Unban,
    UnMute,
    Kick
}

impl std::fmt::Display for TypeModeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TypeModeration {
    fn as_str(&self) -> &'static str {
        match self {
            TypeModeration::Ban => "ban",
            TypeModeration::Mute => "mute",
            TypeModeration::Unban => "unban",
            TypeModeration::UnMute => "unmute",
            TypeModeration::Kick => "kick"
        }
    }
    fn is_sanction(&self) -> bool {
        match self {
            TypeModeration::Ban | TypeModeration::Mute | TypeModeration::Kick => true,
            _ => false
        }
    }
    fn is_a_command(cmd: &str) -> bool {
        match cmd {
            "ban" | "mute" | "unban" | "unmute" | "kick" => true,
            _ => false
        }
    }
}
impl<T: AsRef<str>> From<T> for TypeModeration {
    fn from(s: T) -> Self {
        match s.as_ref() {
            "ban" => TypeModeration::Ban,
            "mute" => TypeModeration::Mute,
            "unban" => TypeModeration::Unban,
            "unmute" => TypeModeration::UnMute,
            "kick" => TypeModeration::Kick,
            _ => panic!("Unknown type of moderation")
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct Action {
    type_mod: TypeModeration,
    guild_id: u64,
    user_id: u64,
    role_id: u64,
}

struct RegistryFile {
    path_file: std::path::PathBuf,
    tasks: RwLock<Vec<task::RegistryTask<Action>>>,
    task_counter: RwLock<task::TaskID>
}

impl RegistryFile {
    async fn from_file(path_file: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let res = Self {
            path_file: path_file.as_ref().to_path_buf(),
            tasks: RwLock::new(Vec::new()),
            task_counter: RwLock::new(1)
        };
        res.load().await?;
        Ok(res)
    }
    async fn save(&self) -> Result<(), String> {
        let mut file = async_std::fs::File::create(&self.path_file).await.map_err(|e| e.to_string())?;
        let tasks = self.tasks.read().await;
        let data = ron::to_string(&*tasks).map_err(|e| e.to_string())?;
        file.write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    async fn load(&self) -> Result<(), String> {
        if self.path_file.exists() {
            let data = std::fs::read_to_string(&self.path_file).map_err(|e| format!("modo RegistryFile: can't open file: {}", e.to_string()))?;
            let tasks = ron::from_str(&data).map_err(|e| format!("modo RegistryFile: can't open file: {}", e.to_string()))?;
            *self.tasks.write().await = tasks;
        }
        Ok(())
    }
}
#[async_trait]
impl task::Registry<Action> for RegistryFile {
    async fn register(&self, task: task::Task<Action>) -> Result<task::TaskID, String> {
        let mut tasks = self.tasks.write().await;
        let id = self.task_counter.read().await.clone();
        tasks.push(task::RegistryTask{
            id,
            task
        });
        *self.task_counter.write().await += 1;
        Ok(id)
    }
    async fn unregister(&self, id: task::TaskID) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id as _);
        Ok(())
    }

    async fn get(&self, id: task::TaskID) -> Option<task::Task<Action>> {
        self.tasks.read().await.iter().find(|v| v.id == id).map(|t| t.task.clone())
    }

    async fn get_all(&self) -> Vec<task::RegistryTask<Action>> {
        self.tasks.read().await.clone()
    }
}

impl Action {
    async fn undo(ctx: Context, action: Action) {
        match action.type_mod {
            TypeModeration::Ban => {
                let username = UserId(action.user_id).to_user(&ctx).await
                    .and_then(|v| Ok(v.name))
                    .or_else::<(), _>(|_| Ok(action.user_id.to_string()))
                    .unwrap();
                match GuildId(action.guild_id).unban(&ctx, action.user_id).await {
                    Ok(_) => println!("Membre \"{}\" débanni", username),
                    Err(e) => println!("Erreur lors du déban de {} : {}", username, e)
                }
            },
            TypeModeration::Mute => {
                let mut member = match GuildId(action.guild_id).member(&ctx, action.user_id).await{
                    Ok(m) => m,
                    Err(e) => {
                        println!("Impossible de trouver le membre {} dans le serveur {}: {}", action.user_id, action.guild_id, e.to_string());
                        return
                    }
                };
                match member.remove_role(&ctx, action.role_id).await {
                    Ok(_) => println!("Membre {} débanni", member.display_name()),
                    Err(e) => println!("Impossible de retirer le rôle {} du membre \"{}\" dans le serveur {}: {}", action.role_id, member.display_name(), action.guild_id, e.to_string())
                };
            },
            _ => ()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct ModerationData {
    mod_until: Vec<Action>,
    muted_role: u64,
}

pub struct Moderation {
    tasks: RwLock<Option<task::TaskManager<Action, RegistryFile>>>,
}
impl Moderation {
    pub fn new() -> Self {
        Moderation {
            tasks: RwLock::new(None)
        }
    }
}

enum Sanction {
    Ban{
        user_id: UserId,
        guild_id: GuildId,
        duration: Option<Duration>,
        historique: u8,
        reason: String,
    },
    Mute {
        user_id: UserId,
        guild_id: GuildId,
        duration: Option<Duration>,
        reason: String,
    },
    Kick {
        user_id: UserId,
        guild_id: GuildId,
        reason: String,
    },
    Unban {
        user_id: UserId,
        guild_id: GuildId,
    },
    Unmute {
        user_id: UserId,
        guild_id: GuildId,
    },
}

impl Sanction {
    pub const fn name(&self) -> &'static str {
        match &self {
            Sanction::Ban{..} => "Ban",
            Sanction::Mute{..} => "Mute",
            Sanction::Kick{..} => "Kick",
            Sanction::Unban{..} => "Unban",
            Sanction::Unmute{..} => "Unmute",
        }
    }
    pub const fn preterite(&self) -> &'static str {
        match &self {
            Sanction::Ban{..} => "banni",
            Sanction::Mute{..} => "mute",
            Sanction::Kick{..} => "kick",
            Sanction::Unban{..} => "débanni",
            Sanction::Unmute{..} => "démute",
        }
    }
    pub async fn apply(&self, ctx: &Context) -> serenity::Result<()> {
        
        match self {
            Sanction::Ban{user_id, guild_id, historique, reason, ..} => {
                guild_id.ban_with_reason(ctx, user_id, *historique, reason).await
            },
            Sanction::Mute{user_id, guild_id, ..} => {
                let role = guild_id
                    .roles(ctx).await?
                    .iter()
                    .find(|(_, r)| r.name == "muted")
                    .map(|(id, _)| *id);
                match role {
                    Some(role) => {
                        let mut member = guild_id.member(ctx, user_id).await?;
                        member.add_role(ctx, role).await?;
                        Ok(())
                    },
                    None => {
                        warn!("Impossible de trouver le rôle \"muted\" dans le serveur {}", guild_id);
                        Err(serenity::Error::Other("Impossible de trouver le rôle \"muted\" dans le serveur"))
                    }
                } 
            },
            Sanction::Kick{user_id, guild_id, reason} => {
                guild_id.kick_with_reason(ctx, user_id, reason).await
            },
            Sanction::Unban{user_id, guild_id} => {
                guild_id.unban(ctx, user_id).await
            },
            Sanction::Unmute{user_id, guild_id} => {
                let role = guild_id
                    .roles(ctx).await?
                    .iter()
                    .find(|(_, r)| r.name == "muted")
                    .map(|(id, _)| *id);
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
        match self {
            Sanction::Ban{user_id, ..} => *user_id,
            Sanction::Mute{user_id, ..} => *user_id,
            Sanction::Kick{user_id, ..} => *user_id,
            Sanction::Unban{user_id, ..} => *user_id,
            Sanction::Unmute{user_id, ..} => *user_id,
        }
    }
    pub fn guild_id(&self) -> GuildId {
        match self {
            Sanction::Ban{guild_id, ..} => *guild_id,
            Sanction::Mute{guild_id, ..} => *guild_id,
            Sanction::Kick{guild_id, ..} => *guild_id,
            Sanction::Unban{guild_id, ..} => *guild_id,
            Sanction::Unmute{guild_id, ..} => *guild_id,
        }
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
    fn to_message<S: ToString>(&self, color: serenity::utils::Colour, description: S) -> message::Message {
        let mut m = message::Message::new();
        m.add_embed(|e| {
            e.title(self.name());
            e.description(description);
            e.color(color);
            match self {
                Sanction::Ban{duration, reason, ..} => {
                    if let Some(duration) = duration {
                        e.field("Durée", format!("{}", duration.to_string()), true);
                    }
                    e.field("Raison", reason, true);
                },
                Sanction::Mute{duration, reason, ..} => {
                    if let Some(duration) = duration {
                        e.field("Durée", format!("{}", duration.to_string()), true);
                    }
                    e.field("Raison", reason, true);
                },
                Sanction::Kick{reason, ..} => {
                    e.field("Raison", reason, true);
                },
                _ => ()
            }
            e
        });
        m
    }
}

#[commands]
impl Moderation {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, _: &ReadyEvent) {
        let mut tasks = self.tasks.write().await;
        let ctx = ctx.clone();
        let task_func = move |d| {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                Action::undo(ctx, d).await
            });
        };
        match &mut *tasks {
            Some(tasks) => tasks.reset_func(task_func),
            None => {
                let registry = RegistryFile::from_file("./data/moderation2.ron").await.unwrap();
                let new_tasks = task::TaskManager::new(task_func, registry);
                *tasks = Some(new_tasks);
            }
        }
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
        let duration = None;
        
        self.do_sanction(ctx, app_cmd, Sanction::Ban{
            user_id: member,
            guild_id,
            reason: raison,
            duration,
            historique: del_msg.map(|v| v.clamp(0, 7)).unwrap_or(0)
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
        self.do_sanction(ctx, app_cmd, Sanction::Kick{
            user_id: member,
            guild_id,
            reason
        }).await;
    }
    #[command(description="Mute un membre du serveur")]
    pub async fn mute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à mute", name="qui")]
        member: UserId,
        #[argument(description="Raison du ban")]
        raison: String,
        // #[argument(description="Durée du mute")]
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
        let duration = None;
        
        self.do_sanction(ctx, app_cmd, Sanction::Mute{
            user_id: member,
            guild_id,
            reason: raison,
            duration,
        }).await;
    }
    #[command(description="Débanni un membre du serveur")]
    pub async fn unban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à débannir", name="qui")]
        member: UserId
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        self.do_sanction(ctx, app_cmd, Sanction::Unban{
            user_id: member,
            guild_id
        }).await;
    }
    #[command(description="Démute un membre du serveur")]
    pub async fn unmute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à démute", name="qui")]
        member: UserId
    ) {
        let guild_id = app_cmd.get_guild_id().unwrap_or(GuildId(0));
        self.do_sanction(ctx, app_cmd, Sanction::Unmute{
            user_id: member,
            guild_id
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
        match &sanction {
            Sanction::Ban { .. } | Sanction::Mute { .. } | Sanction::Kick { .. } => {
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
        
        match resp.send_message(sanction.to_server_message(ctx).await).await{
            Ok(_) => (),
            Err(e) => warn!("Impossible de renvoyer la réponse d'une commande: {}", e.to_string())
        }
    }
}