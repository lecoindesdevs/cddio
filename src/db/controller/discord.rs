use crate::db::{
    model, 
    controller::Error, 
    IDType
};
use crate::{log_info, log_error};
use sea_orm::{entity::*, TransactionTrait};
use std::collections::HashSet;

pub async fn create_user_if_not_exists(db: &sea_orm::DbConn, ctx: &serenity::client::Context, user_id: serenity::model::id::UserId) -> Result<IDType, Error> {
    log_info!("Creating user");
    let db_user_id = match user_id.0.try_into() {
        Ok(v) => v,
        Err(e) => return Err(Error::Custom(format!("Erreur de conversion de l'ID de l'utilisateur: {}", e))),
    };
    if let None = model::discord::User::find_by_id(db_user_id as IDType).one(db).await.map_err(Error::SeaORM)? {
        let user = user_id.to_user(ctx).await.map_err(Error::Serenity)?;
        let avatar_url = user.avatar_url().unwrap_or_default();
        let username = if user.discriminator > 0 {format!("{}#{}", user.name, user.discriminator)} else { user.name };
        let active_model = model::discord::user::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_user_id as IDType),
            name: sea_orm::ActiveValue::Set(username),
            avatar: sea_orm::ActiveValue::Set(avatar_url),
        };
        let res = model::discord::User::insert(active_model).exec(db).await.map_err(Error::SeaORM)?;
        log_info!("User {} added", res.last_insert_id);
    }
    Ok(db_user_id)
}
pub async fn create_channel_if_not_exists(db: &sea_orm::DbConn, ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId) -> Result<IDType, Error> {
    log_info!("Creating channel");
    let db_channel_id = match channel_id.0.try_into() {
        Ok(v) => v,
        Err(e) => return Err(Error::Custom(format!("Erreur de conversion de l'ID du salon: {}", e))),
    };
    if let None = model::discord::Channel::find_by_id(db_channel_id).one(db).await.map_err(Error::SeaORM)? {
        let channel_name = channel_id.name(ctx).await.unwrap_or_default();
        model::discord::Channel::insert(model::discord::channel::ActiveModel{
            id: sea_orm::ActiveValue::Set(db_channel_id),
            name: sea_orm::ActiveValue::Set(channel_name),
        }).exec(db).await.map_err(Error::SeaORM)?;
    }
    Ok(db_channel_id)
}

pub async fn save_channel(db: &sea_orm::DbConn, ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId) -> Result<IDType, Error> {
    use serenity::futures::StreamExt;
    
    let db_chan = create_channel_if_not_exists(db, ctx, channel_id).await?;
    let txn = db.begin().await.map_err(Error::SeaORM)?;
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
                last_modified: sea_orm::ActiveValue::Set(msg.timestamp.unix_timestamp()),
                in_reply_to: sea_orm::ActiveValue::NotSet,
            };
            let res = model::discord::Message::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
            // log_info!("Message {} saved", res.last_insert_id);
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