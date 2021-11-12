//! Module commmun au composant.
//! 
//! Contient notamment des fonctions utiles.

use crate::component::{self as cmp, CommandMatch};
use serenity::http::CacheHttp;
pub use super::super::data::*;
use crate::component::command_parser as cmd;

/// Envoie un message d'erreur qui indique que l'envoyeur n'a pas la permission dans le channel.
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
/// Envoie un message d'erreur dans le channel.
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
/// Retourne vrai s'il sagit d'un message privé au bot
pub fn is_dm(_ctx: &cmp::Context, msg: &cmp::Message) -> bool {
    msg.guild_id.is_none()
}
/// Retourne Ok(vrai) si membre a le role requis pour utiliser la commande.
/// Peut retourner une erreur si la liste des roles du membre n'a pas pû être récupérée.
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
        None => return Ok(false),
    };
    Ok(roles.iter().any(|r| r.name == role))
}

pub async fn try_match<'a>(ctx: &cmp::Context, msg: &'a cmp::Message, group: &'a cmd::Group, args: Vec<&'a str>) -> Result<cmd::matching::Command<'a>, cmp::CommandMatch> {
    match group.try_match(None, &args) {
        Ok(v) => Ok(v),
        Err(cmd::ParseError::NotMatched) => Err(CommandMatch::NotMatched),
        Err(e_parse) => {
            match e_parse {
                cmd::ParseError::ExpectedPath(_) => {
                    match send_error_message(ctx, msg, "La commande que vous avez tapé est un module. Utilisez l'aide pour plus d'informations.").await {
                        Ok(_) => Err(CommandMatch::Error(e_parse.to_string())),
                        Err(e_send) => Err(CommandMatch::Error(e_send.to_string()))
                    }
                },
                e_parse => Err(CommandMatch::Error(e_parse.to_string()))
            }
        }
    }
}