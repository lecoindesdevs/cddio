//! Module commmun au composant.
//! 
//! Contient notamment des fonctions utiles.

pub mod task;
pub mod time_parser;
pub mod data;

#[inline]
pub fn user_fullname(user: &serenity::model::user::User) -> String {
    format!("{}#{:0>4}", user.name, user.discriminator)
}