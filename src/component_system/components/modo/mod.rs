mod sanction;
mod registry_file;

use chrono::{Duration, Utc};
use log::*;
use futures_locks::RwLock;
use opencdd_components::{ApplicationCommandEmbed, message};
use opencdd_macros::commands;
use serenity::{
    client::Context,
    model::{
        id::*,
        event::*
    }
};
use super::utils::{self, task2::Registry};
use super::utils::time_parser as time;
use crate::component_system::components::utils::task;
use self::{
    sanction::{Sanction, SanctionType},
    registry_file::RegistryFile,
};

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
                let mut new_tasks = task::TaskManager::new(registry, ctx.clone());
                new_tasks.init().await;
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
    async fn abort_last_sanction(&self, user_id: UserId, guild_id: GuildId) {
        match 
        {
            let tasks = self.tasks.read().await;
            let reg = tasks
                .as_ref()
                .unwrap()
                .registry()
                .lock().await;
            reg
                .find_one(|v| v.data.user_id == user_id && v.data.guild_id == guild_id).await
                .map(|(id, _)| id)
        } 
        // Some(2)
        {
            Some(v) => {
                info!("Retrait de l'ancienne sanction du membre {}", user_id);
                let mut tasks = self.tasks.write().await;
                match tasks.as_mut().unwrap().remove(v).await {
                    Ok(_) => info!("Sanction retirée"),
                    Err(e) => error!("Impossible de supprimer la sanction: {}", e)
                }
            },
            None => ()
        };
    }
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
        let guild_id = sanction.guild_id();
        self.abort_last_sanction(user_id, guild_id).await;

        match sanction.data() {
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
    #[inline]
    async fn duration_to_datetime(ctx: &Context, app_cmd: &ApplicationCommandEmbed<'_>, duration_str: Option<String>) -> Option<Option<DateTime<Utc>>> {
        let res = duration_str
            .map(|v| time::parse(v))
            .transpose()
            .map(|v| v.map(|v| Utc::now() + Duration::seconds(v as _)));
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                Self::send_error(ctx, app_cmd, format!("Impossible de parser la durée: {}", e)).await;
                None
            }
        }
    }
    async fn send_error<S: ToString>(ctx: &Context, app_cmd: &ApplicationCommandEmbed<'_>, msg: S) {
        match app_cmd.direct_response(ctx, message::error(msg)).await {
            Ok(_) => (),
            Err(e) => error!("Impossible de renvoyer une réponse directe: {}", e)
        }
    }
}