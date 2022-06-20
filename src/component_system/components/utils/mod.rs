//! Module commmun au composant.
//! 
//! Contient notamment des fonctions utiles.

pub mod task2;
pub mod send;
pub mod message;
#[macro_use]
pub mod app_command;
pub mod commands;
pub mod time_parser;

pub use task2 as task;

use crate::component_system::{self as cmp, CommandMatch};
use serenity::http::CacheHttp;
pub use crate::component_system::data::*;
use crate::component_system::command_parser as cmd;
pub use serenity::model::channel::Message;

/// Retourne vrai s'il sagit d'un message privé au bot
pub fn is_dm(_ctx: &cmp::Context, msg: &Message) -> bool {
    msg.guild_id.is_none()
}
/// Retourne Ok(vrai) si membre a le role requis pour utiliser la commande.
/// Peut retourner une erreur si la liste des roles du membre n'a pas pû être récupérée.
pub async fn has_permission(ctx: &cmp::Context, msg: &Message, role: Option<&str>) -> Result<bool, CommandMatch>{
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
    let roles = match member.roles(&ctx.cache) {
        Some(v) => v,
        None => return Ok(false),
    };
    Ok(roles.iter().any(|r| r.name == role))
}
#[inline]
pub fn user_fullname(user: &serenity::model::user::User) -> String {
    format!("{}#{:0>4}", user.name, user.discriminator)
}