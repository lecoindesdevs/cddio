mod sanction;
mod registry_file;
mod log_audit;

use chrono::{Duration, Utc, DateTime};
use crate::{log_error, log_warn, log_info};
use futures_locks::{RwLock, Mutex};
use opencdd_components::{ApplicationCommandEmbed, message};
use opencdd_macros::commands;
use serenity::{
    client::Context,
    model::{
        id::*,
        event::*
    }
};
use super::utils::{
    self, 
    task::Registry,
    time_parser as time,
    task
};
use self::{
    sanction::{Sanction, SanctionType},
    registry_file::RegistryFile,
};

pub struct Moderation {
    tasks: RwLock<Option<task::TaskManager<Sanction, RegistryFile, Context>>>,
    logger: log_audit::Log,
    bot_id: Mutex<UserId>
}
impl Moderation {
    pub fn new() -> Self {
        Moderation {
            tasks: RwLock::new(None),
            logger: log_audit::Log::new("data/moderation.ron"),
            bot_id: Mutex::new(UserId(0))
        }
    }
}

const AUDIT_TIME_THRESHOLD: i64 = 60;

#[commands]
impl Moderation {
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
        *self.bot_id.lock().await = ready.ready.user.id;
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
        match self.push_log(ctx, event.guild_id, event.user.id, 22).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    #[event(GuildBanRemove)]
    async fn on_ban_remove(&self, ctx: &Context, event: &GuildBanRemoveEvent) {
        match self.push_log(ctx, event.guild_id, event.user.id, 23).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    #[event(GuildMemberRemove)]
    async fn on_member_remove(&self, ctx: &Context, event: &GuildMemberRemoveEvent) {
        match self.push_log(ctx, event.guild_id, event.user.id, 20).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    #[event(GuildMemberUpdate)]
    async fn on_member_update(&self, ctx: &Context, event: &GuildMemberUpdateEvent) {
        match self.push_log(ctx, event.guild_id, event.user.id, 20).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    
    
    #[command(name="ban",description="Banni un membre du serveur")]
    async fn com_ban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à bannir", name="qui")]
        member: UserId,
        #[argument(description="Raison du ban")]
        raison: String,
        #[argument(description="Supprimer l'historique du membre (nombre de jours de 0 à 7)", name="historique")]
        del_msg: Option<u8>,
        #[argument(description="Durée du ban")]
        duree: Option<String>
    ) {
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let msg = loop {
            let guild_id = match app_cmd.get_guild_id() {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande doit être executé sur un serveur.".to_string()),
            };
            let user_by = app_cmd.0.user.id;
            let until = match Self::duration_to_datetime(ctx, &app_cmd,  duree).await {
                Some(v) => v,
                None => break message::error("Durée invalide"),
            };
            break match self.ban(ctx, guild_id, member, Some(user_by), raison, until, del_msg).await {
                Ok(v) => v,
                Err(e) => {
                    log_error!("{}", e);
                    message::error(e)
                }
            };
        };
        match resp.send_message(msg).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
        
    }
    #[command(name="kick",description="Expulse un membre du serveur")]
    async fn com_kick(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à expulser", name="qui")]
        member: UserId,
        #[argument(description="Raison de l'expulsion")]
        raison: String
    ) {
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let msg = loop {
            let guild_id = match app_cmd.get_guild_id() {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande doit être executé sur un serveur.".to_string()),
            };
            let user_by = app_cmd.0.user.id;
            break match self.kick(ctx, guild_id, member, Some(user_by), raison).await {
                Ok(v) => v,
                Err(e) => {
                    log_error!("{}", e);
                    message::error(e)
                }
            };
        };
        match resp.send_message(msg).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    
    #[command(name="mute",description="Mute un membre du serveur")]
    async fn com_mute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à mute", name="qui")]
        member: UserId,
        #[argument(description="Raison du ban")]
        raison: String,
        #[argument(description="Durée du mute")]
        duree: Option<String>
    ) {
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let msg = loop {
            let guild_id = match app_cmd.get_guild_id() {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande doit être executé sur un serveur.".to_string()),
            };
            let user_by = app_cmd.0.user.id;
            let until = match Self::duration_to_datetime(ctx, &app_cmd,  duree).await {
                Some(v) => v,
                None => break message::error("Durée invalide"),
            };
            break match self.mute(ctx, guild_id, member, Some(user_by), raison, until).await {
                Ok(v) => v,
                Err(e) => {
                    log_error!("{}", e);
                    message::error(e)
                }
            };
        };
        match resp.send_message(msg).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    #[command(name="unban",description="Débanni un membre du serveur")]
    async fn com_unban(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à débannir", name="qui")]
        member: UserId
    ) {
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let msg = loop {
            let guild_id = match app_cmd.get_guild_id() {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande doit être executé sur un serveur.".to_string()),
            };
            let user_by = app_cmd.0.user.id;
            break match self.unban(ctx, guild_id, member, Some(user_by)).await {
                Ok(v) => v,
                Err(e) => {
                    log_error!("{}", e);
                    message::error(e)
                }
            };
        };
        match resp.send_message(msg).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }
    #[command(name="unmute",description="Démute un membre du serveur")]
    async fn com_unmute(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(description="Membre à démute", name="qui")]
        member: UserId
    ) {
        let resp = match app_cmd.delayed_response(ctx, false).await {
            Ok(resp) => resp,
            Err(e) => {
                log_error!("{}", e);
                return;
            }
        };
        let msg = loop {
            let guild_id = match app_cmd.get_guild_id() {
                Some(guild_id) => guild_id,
                None => break message::error("Cette commande doit être executé sur un serveur.".to_string()),
            };
            let user_by = app_cmd.0.user.id;
            break match self.unmute(ctx, guild_id, member, Some(user_by)).await {
                Ok(v) => v,
                Err(e) => {
                    log_error!("{}", e);
                    message::error(e)
                }
            };
        };
        match resp.send_message(msg).await {
            Ok(_) => {},
            Err(e) => log_error!("{}", e),
        }
    }

}

impl Moderation {
    #[inline]
    pub async fn ban(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, user_by: Option<UserId>, reason: String, until: Option<DateTime<Utc>>, historique: Option<u8>) -> Result<message::Message, String> {
        let sanction = Sanction {
            user_id,
            guild_id,
            user_by: user_by.unwrap_or(ctx.cache.current_user_id()),
            data: SanctionType::Ban{
                reason,
                until,
                historique: historique.map(|v| v.clamp(0, 7)).unwrap_or(0)
            }
        };
        self.do_sanction(ctx, sanction).await
    }
    #[inline]
    pub async fn kick(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, user_by: Option<UserId>, reason: String) -> Result<message::Message, String> {
        let sanction = Sanction {
            user_id,
            guild_id,
            user_by: user_by.unwrap_or(ctx.cache.current_user_id()),
            data: SanctionType::Kick{
                reason
            }
        };
        self.do_sanction(ctx, sanction).await
    }
    #[inline]
    pub async fn mute(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, user_by: Option<UserId>, reason: String, until: Option<DateTime<Utc>>) -> Result<message::Message, String> {
        let sanction = Sanction {
            user_id,
            guild_id,
            user_by: user_by.unwrap_or(ctx.cache.current_user_id()),
            data: SanctionType::Mute{
                reason,
                until
            }
        };
        self.do_sanction(ctx, sanction).await
    }
    #[inline]
    pub async fn unban(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, user_by: Option<UserId>) -> Result<message::Message, String> {
        let sanction = Sanction {
            user_id,
            guild_id,
            user_by: user_by.unwrap_or(ctx.cache.current_user_id()),
            data: SanctionType::Unban
        };
        self.do_sanction(ctx, sanction).await
    }
    #[inline]
    pub async fn unmute(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, user_by: Option<UserId>) -> Result<message::Message, String> {
        let sanction = Sanction {
            user_id,
            guild_id,
            user_by: user_by.unwrap_or(ctx.cache.current_user_id()),
            data: SanctionType::Unmute
        };
        self.do_sanction(ctx, sanction).await
    }
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
                log_info!("Retrait de l'ancienne sanction du membre {}", user_id);
                let mut tasks = self.tasks.write().await;
                match tasks.as_mut().unwrap().remove(v).await {
                    Ok(_) => log_info!("Sanction retirée"),
                    Err(e) => log_error!("Impossible de supprimer la sanction: {}", e)
                }
            },
            None => ()
        };
    }
    async fn do_sanction(&self, ctx: &Context, sanction: Sanction) -> Result<message::Message, String> {
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
                        Err(e) => log_warn!("L'utilisateur {} a été trouvé mais impossible de lui envoyer un message: {}", user_id, e.to_string())
                   }
                }
            }
            _ => ()
        }
        match sanction.apply(ctx).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Impossible d'appliquer la sanction: {}", e.to_string())),
        };
        if let Err(e) = self.logger.push(&sanction).await {
            log_warn!("Impossible d'enregistrer la sanction dans les logs: {}", e.to_string());
        }
        let msg = sanction.to_server_message(ctx).await;
        match sanction {
            Sanction { data: SanctionType::Ban { until: Some(until), .. } | SanctionType::Mute { until: Some(until), .. }, .. } => {
                let mut tasks = self.tasks.write().await;
                let tasks = tasks.as_mut().unwrap();
                if let Err(e) = tasks.add(sanction, until).await {
                    return Err(format!("Impossible d'ajouter la sanction à la liste: {}", e.to_string()))
                }
            },
            _ => ()
        }
        Ok(msg)
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
            Err(e) => log_error!("Impossible de renvoyer une réponse directe: {}", e)
        }
    }
    async fn push_log(&self, ctx: &Context, guild_id: GuildId, user_id: UserId, action_type: u8) -> Result<(), String>{
        let audit = match guild_id.audit_logs(ctx,Some(action_type), Some(user_id), None, Some(1)).await {
            Ok(audit) => audit,
            Err(e) => {
                log_warn!("Impossible de trouver ou récupérer l'audit: {}", e);
                return Ok(());
            }
        };
        let audit_entry = match audit.entries.first() {
            Some(entry) => entry,
            None => {
                log_warn!("No audit entry found in a log event");
                return Ok(())   
            }
        };
        if Utc::now().timestamp() - audit_entry.id.created_at().unix_timestamp() > AUDIT_TIME_THRESHOLD {
            log_warn!("Audit entry is too old");
            return Ok(());
        }
        let data = match action_type {
            22 => SanctionType::Ban{
                until: None,
                historique: 0,
                reason: audit_entry.reason.clone().unwrap_or_default(),
            },
            23 => SanctionType::Unban,
            20 => SanctionType::Kick{
                reason: audit_entry.reason.clone().unwrap_or_default(),
            },
            25 => {
                use serenity::model::guild::audit_log::Change;
                let changes = match &audit_entry.changes {
                    Some(changes) => changes,
                    None => return Err("No changes found for mute in a mute event".into())
                };
                let is_mute = match changes.iter().filter_map(|change| match change {
                    Change::RolesAdded{new: Some(roles), ..} | Change::RolesRemove{old: Some(roles), ..} => {
                        roles.iter().find(|role| role.name == sanction::ROLE_MUTED)
                            .and(Some(matches!(change, Change::RolesAdded{..})))
                    },
                    _ => None
                }).next() {
                    Some(change) => change,
                    None => return Ok(())
                };
                if is_mute {
                    SanctionType::Mute{
                        until: None,
                        reason: audit_entry.reason.clone().unwrap_or_default(),
                    }
                } else {
                    SanctionType::Unmute
                }
            }
            _ => unreachable!()
        };
        if audit_entry.user_id == self.bot_id.lock().await.0 {
            return Ok(());
        }
        self.logger.push(&Sanction{
            user_id,
            guild_id,
            user_by: audit_entry.user_id,
            data
        }).await
    }
}