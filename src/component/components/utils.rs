//! Module commmun au composant.
//! 
//! Contient notamment des fonctions utiles.

use crate::component::{self as cmp, CommandMatch};
use serenity::http::CacheHttp;
pub use crate::component::data::*;
use crate::component::command_parser as cmd;

pub mod send;
pub mod message;
#[macro_use]
pub mod app_command;
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

pub async fn try_match<'a>(ctx: &cmp::Context, msg: &'a cmp::Message, node: &'a cmd::Node, args: Vec<&'a str>) -> Result<cmd::matching::Command<'a>, cmp::CommandMatch> {
    match node.try_match(None, &args) {
        Ok(v) => Ok(v),
        Err(cmd::ParseError::NotMatched) => Err(CommandMatch::NotMatched),
        Err(e_parse) => {
            let msg_parse = e_parse.to_string();
            match send::error(ctx, msg.channel_id, &msg_parse).await {
                Ok(_) => Err(CommandMatch::Error(msg_parse)),
                Err(e_send) => Err(CommandMatch::Error(format!("- {}\n- {}", msg_parse, e_send.to_string())))
            }
        }
    }
}