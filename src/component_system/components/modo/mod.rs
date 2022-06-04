#![allow(lint)]
use std::{sync::Arc, cell::RefCell};
use log::*;

use async_std::io::WriteExt;
use futures::TryFutureExt;
use futures_locks::RwLock;
use opencdd_components::{self as cmp2, ApplicationCommandEmbed, message, message::ToMessage};
use opencdd_macros::commands;
use serenity::{
    client::Context,
    model::{
        id::*,
        event::*
    }, async_trait
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;
use crate::component_system::components::utils::task;

use super::utils::Data;

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
        let data = std::fs::read_to_string(&self.path_file).map_err(|e| format!("modo RegistryFile: can't open file: {}", e.to_string()))?;
        let tasks = ron::from_str(&data).map_err(|e| format!("modo RegistryFile: can't open file: {}", e.to_string()))?;
        *self.tasks.write().await = tasks;
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
    async fn new() -> Self {
        Moderation {
            tasks: RwLock::new(None)
        }
    }
}

#[commands]
impl Moderation {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
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
        // let (mod_until, muted_role, guild_id)= {
        //     let data = self.data.read().await;
        //     let data = data.read();
            
        //     let guild_id = ready.guilds.iter()
        //         .map(|g| g.id)
        //         .next()
        //         .ok_or_else(|| "No guild found".to_string())?;
        //     (data.mod_until.clone(), data.muted_role, guild_id)
        // };
        // if muted_role == 0 {
        //     let role = guild_id.roles(ctx).await
        //         .map_err(|e| format!("Impossible d'obtenir la liste des roles du serveur: {}", e.to_string()))?
        //         .into_iter()
        //         .find(|(_, role)| role.name == "muted")
        //         .ok_or_else(|| "Impossible de trouver le role muted".to_string())?;
        //     self.data.write().await.write().muted_role = role.0.0;
        // }
        // futures::future::join_all(mod_until.into_iter().map(|act| {
        //     self.make_task(ctx.clone(), guild_id, act)
        // })).await;
        // Ok(())
        todo!()
    }
    #[command(description="Banni un membre du serveur")]
    pub async fn ban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à bannir")]
        member: UserId,
        #[argument(description="Raison du ban")]
        reason: String,
        #[argument(description="Durée du ban")]
        duration: Option<String>
    ) {
        let guild_id = match app_cmd.get_guild_id() {
            Some(v) => v,
            None => {
                app_cmd
                    .direct_response(ctx, message::error("Cette fonction n'est disponible que pour un serveur.")).await
                    .map_err(|e| {error!("Impossible de renvoyer la reponse d'une commande: {}", e.to_string()); Ok(())});
                return;
                //println!("Cette fonction n'est disponible que pour un serveur.")
            }
        };
        // guild_id.ban(ctx, member, dmd)

    }
    #[command(description="Expulse un membre du serveur")]
    pub async fn kick(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à expulser")]
        member: UserId,
        #[argument(description="Raison de l'expulsion")]
        reason: String
    ) {
        unimplemented!()
    }
    #[command(description="Mute un membre du serveur")]
    pub async fn mute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à mute")]
        member: UserId,
        #[argument(description="Durée du mute")]
        duration: Option<String>
    ) {
        unimplemented!()
    }
    #[command(description="Débanni un membre du serveur")]
    pub async fn unban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à débannir")]
        member: UserId
    ) {
        unimplemented!()
    }
    #[command(description="Démute un membre du serveur")]
    pub async fn unmute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à démute")]
        member: UserId
    ) {
        unimplemented!()
    }

}