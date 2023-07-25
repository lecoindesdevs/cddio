use crate::db::{
    model, 
    controller::{Error, FileError}, 
    IDType
};
use crate::{log_info, log_error};
use sea_orm::{entity::*, TransactionTrait, ConnectionTrait};
use std::collections::HashSet;

const ATTACHMENTS_PATH: &str = "data/attachments";

pub async fn create_channel_if_not_exists(db: &sea_orm::DbConn, ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId) -> Result<IDType, Error> {
    log_info!("Creating channel");
    let db_channel_id = channel_id.0.try_into().map_err(|_| Error::BadConversionId(channel_id.0))?;
    if let None = model::discord::Channel::find_by_id(db_channel_id).one(db).await.map_err(Error::SeaORM)? {
        let channel_name = channel_id.name(ctx).await.unwrap_or_default();
        model::discord::Channel::insert(model::discord::channel::ActiveModel{
            id: sea_orm::ActiveValue::Set(db_channel_id),
            name: sea_orm::ActiveValue::Set(channel_name),
        }).exec(db).await.map_err(Error::SeaORM)?;
    }
    Ok(db_channel_id)
}

pub async fn save_attachment_file(attachment: serenity::model::channel::Attachment) -> Result<(), FileError> {
    use std::path::Path;
    let bytes = attachment.download().await.map_err(FileError::Serenity)?;
    let attachment_dir = Path::new(ATTACHMENTS_PATH);
    if !attachment_dir.exists() {
        async_std::fs::create_dir_all(attachment_dir).await.map_err(FileError::Io)?;
    }
    let file_extension = Path::new(&attachment.filename).extension().unwrap_or_default().to_string_lossy().to_owned();
    let filename = format!("{}.{}", attachment.id.0, file_extension);
    let attachment_file = attachment_dir.join(filename);
    async_std::fs::write(&attachment_file, bytes).await.map_err(FileError::Io)?;
    Ok(())
}

pub async fn save_attachment<C: ConnectionTrait>(connector: &C, ctx: &serenity::client::Context, message_id: serenity::model::id::MessageId, attachment: serenity::model::channel::Attachment) -> Result<(Option<tokio::task::JoinHandle<Result<(), FileError>>>, IDType), Error> {
    let db_msg_id: IDType = message_id.0.try_into().map_err(|_| Error::BadConversionId(message_id.0))?;
    let db_attachment_id: IDType = attachment.id.0.try_into().map_err(|_| Error::BadConversionId(attachment.id.0))?;
    if let None = model::discord::Attachment::find_by_id(db_attachment_id).one(connector).await.map_err(Error::SeaORM)? {
        let attachment_url = attachment.url.clone();
        let join_handle = tokio::spawn(async move {save_attachment_file(attachment).await});
        let active_model = model::discord::attachment::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_attachment_id),
            message_id: sea_orm::ActiveValue::Set(db_msg_id),
            url: sea_orm::ActiveValue::Set(attachment_url),
            ..Default::default()
        };
        model::discord::Attachment::insert(active_model).exec(connector).await.map_err(Error::SeaORM)?;
        Ok((Some(join_handle), db_attachment_id))
    } else {
        Ok((None, db_attachment_id))
    }
}

#[inline]
pub async fn save_user_from_id<C: ConnectionTrait>(connector: &C, ctx: &serenity::client::Context, user_id: serenity::model::id::UserId) -> Result<IDType, Error> {
    let user = user_id.to_user(ctx).await.map_err(Error::Serenity)?;
    save_user(connector, user).await
}
pub async fn save_user<C: ConnectionTrait>(connector: &C, user: serenity::model::user::User) -> Result<IDType, Error> {
    let db_user_id: IDType = user.id.0.try_into().map_err(|_| Error::BadConversionId(user.id.0))?;
    if let None = model::discord::User::find_by_id(db_user_id).one(connector).await.map_err(Error::SeaORM)? {
        let active_model = model::discord::user::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_user_id),
            name: sea_orm::ActiveValue::Set(format!("{}#{}", user.name, user.discriminator)),
            avatar: sea_orm::ActiveValue::Set(user.avatar_url().unwrap_or_default()),
        };
        let res = model::discord::User::insert(active_model).exec(connector).await.map_err(Error::SeaORM)?;
        log_info!("User {} added", res.last_insert_id);
    }
    Ok(db_user_id)
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
                save_attachment(&txn, ctx, msg.id, attachment).await?;
            }
        }
    }
    txn.commit().await.map_err(Error::SeaORM)?;
    log_info!("Channel {} saved", db_chan);
    Ok(db_chan)
}