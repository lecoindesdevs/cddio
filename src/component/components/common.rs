use crate::component::{self as cmp, CommandMatch};
use serenity::http::CacheHttp;

pub async fn send_no_perm(_ctx: &cmp::Context, msg: &cmp::Message) -> serenity::Result<()> {
    match msg.channel_id.send_message(&_ctx.http, |m|
        m.embed(|embed| {
            embed
                .title("Attention")
                .description(format!("Vous n'avez pas la permission d'utiliser cette commande"))
                .color(0xFF0000)
        })
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
pub async fn send_error_message<S: AsRef<str>>(_ctx: &cmp::Context, msg: &cmp::Message, error_message: S) -> serenity::Result<()> {
    match msg.channel_id.send_message(&_ctx.http, |m|
        m.embed(|embed| {
            embed
                .title("Attention")
                .description( error_message.as_ref() )
                .color(0xFF0000)
        })
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn is_dm(_ctx: &cmp::Context, msg: &cmp::Message) -> bool {
    msg.guild_id.is_none()
}
pub async fn has_permission(ctx: &cmp::Context, msg: &cmp::Message, role: Option<&str>) -> Result<bool, CommandMatch>{
    let role = match role {
        Some(v) => v,
        None => return Ok(true)
    };
    let guild_id = match msg.guild_id {
        Some(v) => v,
        None => return Ok(false), // Par défaut pour le moment, on ne peut pas utiliser les permissions dans les DM
    };
    let member = guild_id.member(ctx.http(), msg.author.id).await 
        .map_err(|e| CommandMatch::Error(e.to_string()))?;
    let roles = match member.roles(&ctx.cache).await {
        Some(v) => v,
        None => todo!(),
    };
    Ok(roles.iter().any(|r| r.name == role))
}