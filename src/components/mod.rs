//! Module comportant les composants

pub mod misc;
pub use misc::*;
pub mod help;
pub use help::*;
pub mod tickets;
pub use tickets::*;
pub mod slash;
pub use slash::*;
pub mod modo;
pub use modo::*;
pub mod autobahn;
pub use autobahn::*;
pub mod dalle_mini;
pub use dalle_mini::*;

use crate::{log_error, log_info};

// Fonctions utiles pour les composants
mod utils;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    SeaORM(sea_orm::DbErr),
    Custom(String),
}

pub async fn save_ticket(ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId, db: &sea_orm::DbConn) -> Result<(), Error> {
    
    use crate::db;
    use sea_orm::{entity::*, prelude::*, TransactionTrait};
    use serenity::futures::StreamExt;
    
    use crate::{log_error};
    let channel = match channel_id.to_channel(ctx).await.map_err(Error::Serenity)? {
        serenity::model::channel::Channel::Guild(channel) => channel,
        _ => return Err(Error::Custom("Invalid channel".to_string()))
    };

    let txn = db.begin().await.map_err(Error::SeaORM)?;

    let db_chan = {
        let active_model = db::discord::channel::ActiveModel {
            id: sea_orm::ActiveValue::Set(channel.id.0),
            name: sea_orm::ActiveValue::Set(channel.name),
        };
        let res = db::discord::Channel::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
        log_info!("Channel {} saved", res.last_insert_id);
        res.last_insert_id
    };

    let mut messages = channel_id.messages_iter(ctx).boxed();
    while let Some(message_result) = messages.next().await {
        let msg = match message_result {
            Ok(m) => m,
            Err(e) => {
                log_error!("Error while saving ticket: {}", e);
                continue;
            } 
        };
        let user_id = msg.author.id.0;
        if let None = db::discord::User::find_by_id(user_id).one(&txn).await.map_err(Error::SeaORM)? {
            let user = msg.author;
            let active_model = db::discord::user::ActiveModel {
                id: sea_orm::ActiveValue::Set(user.id.0),
                name: sea_orm::ActiveValue::Set(format!("{}#{}", user.name, user.discriminator)),
                avatar: sea_orm::ActiveValue::Set(user.avatar_url().unwrap_or_default()),
            };
            let res = db::discord::User::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            log_info!("User {} added", res.last_insert_id);
        }
        let db_msg = {
            let active_model = db::discord::message::ActiveModel {
                id: sea_orm::ActiveValue::Set(msg.id.0),
                channel_id: sea_orm::ActiveValue::Set(db_chan),
                user_id: sea_orm::ActiveValue::Set(user_id),
                content: sea_orm::ActiveValue::Set(msg.content),
                timestamp: sea_orm::ActiveValue::Set(msg.timestamp.unix_timestamp()),
                in_reply_to: sea_orm::ActiveValue::NotSet,
            };
            let res = db::discord::Message::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            log_info!("Message {} saved", res.last_insert_id);
            res.last_insert_id
        };
        if !msg.attachments.is_empty() {
            for attachment in msg.attachments {
                let active_model = db::discord::attachment::ActiveModel {
                    message_id: sea_orm::ActiveValue::Set(db_msg),
                    url: sea_orm::ActiveValue::Set(attachment.url),
                    ..Default::default()
                };
                db::discord::Attachment::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            }
        }
    }


    txn.commit().await.map_err(Error::SeaORM)?;
    Ok(())
}