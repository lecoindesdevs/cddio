mod time;
use crate::component_system::{self as cmp, command_parser as cmd};
use chrono::{DateTime, Utc};
use futures_locks::RwLock;
use serde::{Deserialize, Serialize};
use serenity::{async_trait, client::Context};
use serenity::model::{interactions::application_command::ApplicationCommandInteraction, id::{ApplicationId, GuildId}, event::ReadyEvent, prelude::*};
use super::utils::{app_command::{ApplicationCommandEmbed, get_argument}, Data, message};
use tokio::sync::oneshot::Sender;
use super::utils;

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
impl From<TypeModeration> for &'static str {
    fn from(t: TypeModeration) -> Self {
        t.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Action {
    type_mod: TypeModeration,
    user_id: u64,
    time: i64,
}

impl Action {
    fn new(type_mod: TypeModeration, user_id: u64, time: i64) -> Self { Self { type_mod, user_id, time } }
}


#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct ModerationData {
    mod_until: Vec<Action>,
    muted_role: u64,
}
#[derive(Debug)]
pub struct Moderation {
    node: cmd::Node,
    app_id: ApplicationId,
    data: RwLock<Data<ModerationData>>,
    tasks: RwLock<Vec<(UserId, TypeModeration, Sender<()>)>>,
}

struct ModerateParameters {
    pub guild_id: GuildId,
    pub user_id: UserId,
    pub type_mod: TypeModeration,
    pub user_by: UserId,
    pub reason: Option<String>,
    pub duration: Option<u64>,
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

fn format_username(user: &User) -> String {
    format!("{}#{:0>4}", user.name, user.discriminator)
}

impl Moderation {
    pub fn new(app_id: ApplicationId) -> Moderation {
        let ban = cmd::Command::new("ban")
            .set_help("Bannir un membre du serveur. Temporaire si le parametre *pendant* est renseigné.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à bannir")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pourquoi")
                .set_value_type(cmd::ValueType::String)
                .set_help("La raison du ban")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pendant")
                .set_value_type(cmd::ValueType::String)
                .set_help("Pendant combien de temps")
            );
        let mute = cmd::Command::new("mute")
            .set_help("Attribue le rôle *muted* à un membre. Temporaire si le parametre *pendant* est renseigné.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à mute")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pourquoi")
                .set_value_type(cmd::ValueType::String)
                .set_help("La raison du mute")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pendant")
                .set_value_type(cmd::ValueType::String)
                .set_help("Pendant combien de temps")
            );
        let kick = cmd::Command::new("kick")
            .set_help("Expulser un membre du serveur.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à expulser")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("pourquoi")
                .set_value_type(cmd::ValueType::String)
                .set_help("La raison de l'expulsion")
                .set_required(false)
            );
        let unban = cmd::Command::new("unban")
            .set_help("Unban un membre")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à unban")
                .set_required(true)
            );
        let unmute = cmd::Command::new("unmute")
            .set_help("Retire le rôle *muted* à un membre.")
            .add_param(cmd::Argument::new("qui")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à unmute")
                .set_required(true)
            );
        let node = cmd::Node::new()
            .add_command(ban)
            .add_command(mute)
            .add_command(kick)
            .add_command(unban)
            .add_command(unmute);
        Moderation {
            node,
            app_id,
            data: match Data::from_file_default("moderation") {
                Ok(data) => RwLock::new(data),
                Err(e) => panic!("Data moderation: {:?}", e)
            },
            tasks: RwLock::new(Vec::new()),
        }
    }
    // region: discord interface
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        use serenity::model::interactions::Interaction::*;
        use cmp::Event::*;
        match evt {
            Ready(ReadyEvent{ready, ..}) => self.on_ready(ctx, ready).await,
            InteractionCreate(InteractionCreateEvent{interaction: ApplicationCommand(c), ..}) => self.on_applications_command(ctx, c).await,
            GuildBanAdd(GuildBanAddEvent{guild_id, user, ..})  => self.on_discord_action(ctx, guild_id.clone(), user, "ban").await,
            GuildBanRemove(GuildBanRemoveEvent{guild_id, user, ..}) => self.on_discord_action(ctx, guild_id.clone(), user, "unban").await,
            GuildMemberRemove(GuildMemberRemoveEvent{guild_id, user, ..}) => self.on_discord_action(ctx, guild_id.clone(), user, "kick").await,
            _ => Ok(()),
        }
    }
    async fn on_ready(&self, ctx: &cmp::Context, ready: &serenity::model::gateway::Ready) -> Result<(), String> {
        
        let (mod_until, muted_role, guild_id)= {
            let data = self.data.read().await;
            let data = data.read();
            
            let guild_id = ready.guilds.iter()
                .map(|g| g.id())
                .next()
                .ok_or_else(|| "No guild found".to_string())?;
            (data.mod_until.clone(), data.muted_role, guild_id)
        };
        if muted_role == 0 {
            let role = guild_id.roles(ctx).await
                .map_err(|e| format!("Impossible d'obtenir la liste des roles du serveur: {}", e.to_string()))?
                .into_iter()
                .find(|(_, role)| role.name == "muted")
                .ok_or_else(|| "Impossible de trouver le role muted".to_string())?;
            self.data.write().await.write().muted_role = role.0.0;
        }
        futures::future::join_all(mod_until.into_iter().map(|act| {
            self.make_task(ctx.clone(), guild_id, act)
        })).await;
        Ok(())
    }
    async fn on_applications_command(&self, ctx: &Context, app_command: &ApplicationCommandInteraction) -> Result<(), String> {
        if app_command.application_id != self.app_id {
            // La commande n'est pas destiné à ce bot
            return Ok(());
        }
        let app_cmd = ApplicationCommandEmbed::new(app_command);
        let command_name = app_cmd.fullname();
        if !TypeModeration::is_a_command(command_name.as_str()) {
            return Ok(());
        }
        let guild_id = match app_cmd.get_guild_id() {
            Some(v) => v,
            None => return Err("Vous devez être dans un serveur pour utiliser cette commande.".into())
        };
        

        let userby_id = app_cmd.0.member.as_ref().ok_or_else(|| "Impossible de récumérer le membre qu'a fait la commande.")?.user.id;

        let params = ModerateParameters{
            guild_id,
            user_id: get_argument!(app_cmd, "qui", User)
                .map(|v| v.0)
                .ok_or_else(|| "Vous devez mentionner un membre.".to_string())
                .and_then(|user| if user.id != userby_id {
                    Ok(user.id)
                } else {
                    Err(format!("Vous ne pouvez pas vous {} vous-même.", command_name))
                })?,
            type_mod: command_name.into(),
            user_by: userby_id,
            reason: get_argument!(app_cmd, "pourquoi", String).cloned(),
            duration: match get_argument!(app_cmd, "pendant", String) {
                Some(v) => Some(time::parse(v).map_err(|e| format!("Paramètre pendant: Impossible de parser la durée: {}", e))?),
                None => None
            },
        };
        let msg = self.moderate(ctx, params).await?;
        app_command.create_interaction_response(ctx, |resp|{
            *resp = msg.into();
            resp
        }).await.map_err(|e| format!("Cannot create response: {}", e))
    }
    async fn on_discord_action(&self, ctx: &Context, guild_id: GuildId, target_user: &User, what: &str) -> Result<(), String> {
        const ACTION_TYPES: [(&str, u8); 3] = [("ban", 22), ("unban", 23), ("kick", 20)];
        let audit_logs = guild_id.audit_logs(ctx, 
            ACTION_TYPES.iter().find(|(name, _)| *name == what).map(|(_, id)| *id),
            None,
            None,
            Some(5)
        ).await.map_err(|e| format!("Impossible d'obtenir les logs d'audit: {}", e.to_string()))?;
        let found = audit_logs.entries.into_iter()
            .find(|(_, entry)| entry.target_id.unwrap_or_default() == target_user.id.0);
        if found.is_none() {
            return Ok(());
        }
        let entry = found.map(|(_, v)| v).unwrap();
        let username = entry.user_id
            .to_user(ctx).await
            .and_then(|u| Ok(format_username(&u)))
            .or_else(|e| Err(format!("Impossible d'obtenir le nom de l'utilisateur: {}", e.to_string())))?;
        let target_username = format_username(&target_user);
        Self::write_log(
            &target_username,
            &username,
            what,
            entry.reason.as_ref().map(|r| r.as_str()),
            None
        ).await;
        Ok(())
    }
    // endregion: discord interface
    // region: tasks
    async fn task(ctx: Context, guild_id: GuildId, action: Action, data: RwLock<Data<ModerationData>>) {
        let time_point = DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(action.time, 0), Utc);
        let duration = time_point - chrono::Utc::now();
        if duration.num_seconds()>0 { 
            tokio::time::sleep(duration.to_std().unwrap()).await;
        }
        let action_done = match action.type_mod {
            TypeModeration::Mute => {
                let mut member = match guild_id.member(&ctx, action.user_id).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Impossible d'avoir le membre {}: {}", action.user_id, e);
                        return;
                    }
                };
                
                let muted_role = {data.read().await.read().muted_role};
                member.remove_role(&ctx, muted_role).await
            },
            TypeModeration::Ban => guild_id.unban(&ctx, action.user_id).await,
            _ => return,
        };
        let username = UserId(action.user_id).to_user(&ctx).await.map(|user| format!("{} ({})", format_username(&user), action.user_id)).unwrap_or_else(|_| action.user_id.to_string());
        if let Err(e) = action_done {
            eprintln!("modo::task erreur {}: {}", username, e.to_string());
        } else { 
            println!("modo::task: Sanction contre {} retiré", username);
            let mut data = data.write().await;
            let mut data = data.write();
            let mod_until = &mut data.mod_until;
            
            match mod_until.iter()
                .position(|Action{user_id, ..}| user_id == &action.user_id)
                .map(|idx| mod_until.remove(idx))
                {
                    Some(_) => (),
                    None => eprintln!("modo::task: sanction non trouvée dans les données pour l'utilisateur {}", username)
                };
        }
    }
    async fn make_task(&self, ctx: Context, guild_id: GuildId, action: Action) {
        let who = match guild_id.member(&ctx, action.user_id).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Impossible d'avoir le membre {}: {}", action.user_id, e);
                return;
            }
        };
        let task = Self::task(ctx, guild_id, action.clone(), self.data.clone());
        let (stop_task, stop_me) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            tokio::select! {
                _ = task => println!("{} du membre {} fini", action.type_mod, who.display_name()),
                _ = stop_me => println!("Arrêt {} temporaire de {}", action.type_mod, who.display_name()),
            }
        });
        self.tasks.write().await.push((UserId(action.user_id), action.type_mod, stop_task)); 
    }
    async fn remove_task(&self, who: UserId, type_mod: TypeModeration) {
        let mut tasks = self.tasks.write().await;
        let idx = match tasks.iter().position(|(user_id, t, _)| user_id == &who && t == &type_mod) {
            Some(idx) => idx,
            None => return
        };
        let (_, _, stop_task) = tasks.remove(idx);
        stop_task.send(()).unwrap_or(());
    }
    async fn add_until(&self, who: u64, when: i64, what: TypeModeration) -> Action {
        let mut data = self.data.write().await;
        let mut data = data.write();
        let result = Action::new(what, who, when);
        data.mod_until.push(result.clone());
        result
    }
    async fn remove_until(&self, who: u64, what: TypeModeration) {
        let mut data = self.data.write().await;
        let mut data = data.write();
        data.mod_until
            .iter()
            .position(|a| a.user_id == who && a.type_mod == what)
            .map(|idx| {data.mod_until.remove(idx);})
            .unwrap_or_default();
    }
    // endregion
    // region: actions
    async fn moderate(&self, ctx: &Context, params: ModerateParameters) -> Result<message::Message, String>
    {
        let time = match &params.duration {
            Some(v) => {
                let duration = chrono::Duration::seconds(*v as _);
                let time_point = chrono::Local::now() + duration;
                let duration_str = time::format_duration(*v);
                Some((time_point.timestamp(), time_point, duration_str))
            }
            _ => None
        };
        let muted_role = if matches!(params.type_mod, TypeModeration::Mute | TypeModeration::UnMute) {
            let muted_role = self.data.read().await.read().muted_role;
            if muted_role == 0 {
                return Err("Le rôle de mute n'est pas défini.".into());
            }
            Some(RoleId(muted_role))
        } else {
            None
        };
        'check: loop {
        let mut roles: [usize;2] = [0, 0];
        let mut riter = roles.iter_mut();
            let guild_roles = match params.guild_id.roles(&ctx).await {
                Ok(v) => v,
                Err(e) => return Err(format!("Impossible d'avoir les rôles du serveur {}: {}", params.guild_id.0, e))
            };
        for user in [params.user_id, params.user_by] {
                let userroles = params.guild_id
                    .member(ctx, user).await
                    .or_else (|e| Err(format!("Impossible de récupérer un membre du serveur: {}", e)))?
                    .roles(ctx).await;
                let top_role = match userroles {
                    Some(roles) if roles.is_empty() => break 'check,
                    Some(roles) => roles[0].id,
                    None => break 'check,
                };
                let pos_role_user = guild_roles.iter()
                    .position(|r| *r.0 == top_role)
                    .unwrap_or(0);
            if let Some(r) = riter.next() {
                *r = pos_role_user;
            }
        }
            if roles[0] <= roles[1] {
                return Err("Le membre à modérer a un rôle plus élevé ou égal au rôle du modérateur.".into());
            }
            break;
        }
        let user = params.user_id.to_user(&ctx).await.or_else(|_| Err("Impossible de trouver l'utilisateur.".to_string()))?;
        if params.type_mod.is_sanction() {
            let when = time.as_ref().map(|(_, when, _)| when.format("%d/%m/%Y à %H:%M:%S").to_string());
            match self.warn_member(
                ctx, 
                &user,
                params.type_mod.into(), 
                when.as_ref().map(|v| v.as_str()), 
                params.reason.as_ref().map(|v| v.as_str()), 
                params.guild_id.as_ref().name(ctx).await.unwrap().as_str()
            ).await{
                Err(e) => println!("[WARN] Impossible d'avertir le membre: {}", e),
                _ => ()
            }
        }
        Self::do_action(
            ctx,
            &params,
            muted_role,
        ).await
            .map_err(|e| format!("Impossible de {} le membre: {}", params.type_mod.as_str(), e))?;
        
        
        tokio::join!(
            self.remove_task(params.user_id, params.type_mod),
            self.remove_until(params.user_id.0, params.type_mod)
        );
        
        let username = format!("{} (<@{}>)", format_username(&user), params.user_id);
        let who_did = format_username(&params.user_by.to_user(&ctx).await.unwrap());
        
        Self::write_log(
            &username, 
            &who_did, 
            params.type_mod.as_str(), 
            params.reason.as_ref().map(|v| v.as_str()), 
            time.as_ref().map(|v| v.2.as_str()),
        ).await;

        let mut msg = message::success(format!("{} a été {}.", username, params.type_mod.as_str()));
        if let Some(embed) = msg.last_embed_mut() {
            if let Some(reason) = params.reason {
                embed.field("Raison", reason, false);
            }
            if let Some((timestamp, datetime, duration)) = time {
                self.make_task(ctx.clone(), params.guild_id, self.add_until(user.id.0, timestamp, params.type_mod).await).await;
                embed.field("Pendant", duration, false);
                embed.field("Prend fin", datetime.format("%d/%m/%Y à %H:%M:%S").to_string(), true);
            }
        }
        Ok(msg)
    }
    async fn warn_member(&self, ctx: &Context, user: &User, keyword: &str, when: Option<&str>, reason: Option<&str>, guild_name: &str) -> Result<(), String> {
        match user.direct_message(ctx, |msg| {
            let mut msg_content = if let Some(when) = when {
                format!("Vous avez été temporairement **{}** du serveur {}.\n__Prend fin le__ : {}", keyword, guild_name, when)
            } else {
                format!("Vous avez été **{}** du serveur {}.", keyword, guild_name)
            };
            if let Some(reason) = reason {
                msg_content = format!("{}\n__Raison__ : {}", msg_content, reason);
            }
            msg.content(msg_content);
            msg
        }).await {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(format!("Impossible d'envoyer le message de bannissement à l'utilisateur {}: {}", format_username(user), e))
            }
        }
    }
    async fn do_action(
        ctx: &Context,
        params: &ModerateParameters,
        muted_role: Option<RoleId>,
    ) -> serenity::Result<()> 
    {
        match (params.type_mod, &params.reason) {
            (TypeModeration::Ban, Some(reason)) => params.guild_id.ban_with_reason(&ctx, params.user_id, 0, reason).await?,
            (TypeModeration::Ban, None) => params.guild_id.ban(&ctx, params.user_id, 0).await?,
            (TypeModeration::Mute, _) => {
                if let Some(muted_role) = muted_role {
                    let mut member = params.guild_id.member(ctx, params.user_id).await?;
                    member.add_role(ctx, muted_role).await?;
                }
            },
            (TypeModeration::Kick, Some(reason)) => params.guild_id.kick_with_reason(&ctx, params.user_id, reason.as_str()).await?,
            (TypeModeration::Kick, None) => params.guild_id.kick(&ctx, params.user_id).await?,
            (TypeModeration::Unban, _) => params.guild_id.unban(&ctx, params.user_id).await?,
            (TypeModeration::UnMute, _) => {
                if let Some(muted_role) = muted_role {
                    let mut member = params.guild_id.member(ctx, params.user_id).await?;
                    member.remove_role(ctx, muted_role).await?;
                }
            },
        };
        Ok(())
    }
    // endregion
    async fn write_log(who: &str, who_did: &str, what: &str, why:Option<&str>, during: Option<&str>)
    {
        use tokio::fs::OpenOptions;
        use std::io::Write;
        let file_path = utils::DATA_DIR.join("modo.log");
        let file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path).await {
                Ok(v) => v,
                Err(e) => {
                    println!("Impossible d'ouvrir le fichier de log: {}", e);
                    return;
                }
            };
        let now = chrono::Local::now();
        let mut file: std::fs::File = file.into_std().await;
        let file = &mut file;
        match (|| -> std::io::Result<()>{
            write!(file, "{:=<10}\nWhen: {}\nWho: {}\nWhat: {}\nBy: {}\n", "", now.to_rfc3339(), who, what, who_did)?;
            if let Some(why) = why {
                write!(file, "Why: {}\n", why)?;
            }
            if let Some(during) = during {
                write!(file, "During: {}\n", during)?;
            }
            Ok(())
        })() {
            Ok(_) => (),
            Err(e) => {
                println!("Impossible d'écrire dans le fichier de log: {}", e);
                return;
            }
        }
    }
    // region Actions throught other components
    #[allow(dead_code)]
    pub async fn mute(&self, ctx: &Context, guild_id: GuildId, user: UserId, reason: Option<String>, time: Option<chrono::Duration>) -> Result<message::Message, String> {
        let params = ModerateParameters {
            guild_id,
            user_id: user,
            user_by: ctx.cache.current_user().await.id,
            type_mod: TypeModeration::Mute,
            reason,
            duration: time.map(|time| (time.num_seconds() as u64)),
        };
        
        match self.moderate(ctx, params).await {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
    #[allow(dead_code)]
    pub async fn ban(&self, ctx: &Context, guild_id: GuildId, user: UserId, reason: Option<String>, time: Option<chrono::Duration>) -> Result<message::Message, String> {
        let params = ModerateParameters {
            guild_id,
            user_id: user,
            user_by: ctx.cache.current_user().await.id,
            type_mod: TypeModeration::Ban,
            reason,
            duration: time.map(|time| (time.num_seconds() as u64)),
        };
        
        match self.moderate(ctx, params).await {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
    #[allow(dead_code)]
    pub async fn kick(&self, ctx: &Context, guild_id: GuildId, user: UserId, reason: Option<String>) -> Result<message::Message, String> {
        let params = ModerateParameters {
            guild_id,
            user_id: user,
            user_by: ctx.cache.current_user().await.id,
            type_mod: TypeModeration::Kick,
            reason,
            duration: None,
        };
        
        match self.moderate(ctx, params).await {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
    // endregion
}
