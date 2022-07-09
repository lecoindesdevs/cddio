//! Module commmun au composant.
//! 
//! Contient notamment des fonctions utiles.

pub mod task;
pub mod time_parser;
pub mod data;

use crate::component_system::{self as cmp};
pub use serenity::model::channel::Message;

/// Retourne vrai s'il sagit d'un message privé au bot
pub fn is_dm(_ctx: &cmp::Context, msg: &Message) -> bool {
    msg.guild_id.is_none()
}
#[inline]
pub fn user_fullname(user: &serenity::model::user::User) -> String {
    format!("{}#{:0>4}", user.name, user.discriminator)
}