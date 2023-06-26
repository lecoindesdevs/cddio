use crate::db::{
    model, 
    controller::Error, 
    IDType
};
use crate::{log_info, log_error};
use sea_orm::{entity::*, TransactionTrait};
use std::collections::HashSet;

pub async fn save_channel(db: &sea_orm::DbConn, ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId) -> Result<IDType, Error> {
    use serenity::futures::StreamExt;
    
    let channel = match channel_id.to_channel(ctx).await.map_err(Error::Serenity)? {
        serenity::model::channel::Channel::Guild(channel) => channel,
        _ => return Err(Error::Custom("Invalid channel".to_string()))
    };

    let txn = db.begin().await.map_err(Error::SeaORM)?;

    let db_chan = {
        let active_model = model::discord::channel::ActiveModel {
            id: sea_orm::ActiveValue::Set(channel.id.0 as IDType),
            name: sea_orm::ActiveValue::Set(channel.name),
        };
        let res = model::discord::Channel::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
        
        res.last_insert_id
    };
    let mut registered_users = HashSet::new();
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
        if !registered_users.contains(&user_id) {
            if let None = model::discord::User::find_by_id(user_id as IDType).one(&txn).await.map_err(Error::SeaORM)? {
                let user = msg.author;
                let active_model = model::discord::user::ActiveModel {
                    id: sea_orm::ActiveValue::Set(user.id.0 as IDType),
                    name: sea_orm::ActiveValue::Set(format!("{}#{}", user.name, user.discriminator)),
                    avatar: sea_orm::ActiveValue::Set(user.avatar_url().unwrap_or_default()),
                };
                let res = model::discord::User::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
                log_info!("User {} added", res.last_insert_id);
            }
            registered_users.insert(user_id);
        }
        let db_msg = {
            let active_model = model::discord::message::ActiveModel {
                id: sea_orm::ActiveValue::Set(msg.id.0 as IDType),
                channel_id: sea_orm::ActiveValue::Set(db_chan),
                user_id: sea_orm::ActiveValue::Set(user_id as IDType),
                content: sea_orm::ActiveValue::Set(msg.content),
                timestamp: sea_orm::ActiveValue::Set(msg.timestamp.unix_timestamp()),
                in_reply_to: sea_orm::ActiveValue::NotSet,
            };
            let res = model::discord::Message::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            log_info!("Message {} saved", res.last_insert_id);
            res.last_insert_id
        };
        if !msg.attachments.is_empty() {
            for attachment in msg.attachments {
                let active_model = model::discord::attachment::ActiveModel {
                    message_id: sea_orm::ActiveValue::Set(db_msg),
                    url: sea_orm::ActiveValue::Set(attachment.url),
                    ..Default::default()
                };
                model::discord::Attachment::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            }
        }
    }
    txn.commit().await.map_err(Error::SeaORM)?;
    log_info!("Channel {} saved", db_chan);
    Ok(db_chan)
}