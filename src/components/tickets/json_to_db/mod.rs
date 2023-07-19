mod archive;
mod error;
use std::{
    fs::{
        File,
        read_dir,
    },
    io::BufReader, ffi::OsStr, path::PathBuf,
};
use sea_orm::{
    DatabaseConnection,
    prelude::*,
    ActiveValue
};
use crate::{
    db::{
        model,
        IDType,
    },
};


struct Migration;

const TICKET_PATH: &str = "./data/tickets";
const ARCHIVE_PATH: &str = "./data/tickets/archives";

async fn from_category(db: &DatabaseConnection, category: archive::Category) -> error::CategoryResult<IDType> {
    let discord_category_id: IDType = category.id.try_into().map_err(|_| error::CategoryError::BadID(category.id))?;
    let active_model = model::ticket::category::ActiveModel {
        name: sea_orm::ActiveValue::Set(category.name),
        prefix: sea_orm::ActiveValue::Set(category.prefix),
        discord_category_id: sea_orm::ActiveValue::Set(discord_category_id),
        description: sea_orm::ActiveValue::Set(category.desc),
        hidden: sea_orm::ActiveValue::Set(category.hidden),
        .. Default::default()
    };
    let res = model::ticket::Category::insert(active_model).exec(db).await;
    match res {
        Ok(v) => Ok(v.last_insert_id),
        Err(e) => Err(error::CategoryError::SeaORM(e)),
    }
}


async fn from_categories(db: &DatabaseConnection, categories: Vec<archive::Category>) -> error::CategoriesResult<IDType> {
    let mut results = error::MultiResult::new();
    for category in categories {
        results.push(from_category(db, category).await);
    }
    results
}

async fn from_user(db: &DatabaseConnection, user: archive::ArchiveUser) -> error::UserResult<Option<IDType>> {
    let db_user_id: IDType = user.id.try_into().map_err(|_| error::UserError::BadID(user.id))?;
    match model::discord::User::find_by_id(db_user_id).one(db).await {
        Ok(None) => return Ok(None),
        Err(e) => return Err(error::UserError::SeaORM(e)),
        _ => ()
    }
    let res = model::discord::User::insert(
        model::discord::user::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_user_id),
            name: sea_orm::ActiveValue::Set(user.name),
            avatar: sea_orm::ActiveValue::Set(user.avatar),
        }
    ).exec(db).await;
    match res {
        Ok(v) => Ok(Some(v.last_insert_id)),
        Err(e) => Err(error::UserError::SeaORM(e)),
    }
}
async fn from_users(db: &DatabaseConnection, users: Vec<archive::ArchiveUser>) -> error::UsersResult<Option<IDType>> {
    let mut results = error::MultiResult::new();
    for user in users {
        results.push(from_user(db, user).await);
    }
    results
}

async fn from_attachment(db: &DatabaseConnection, attachment: String, message_id: IDType) -> Result<IDType, DbErr> {
    model::discord::Attachment::insert(
        model::discord::attachment::ActiveModel {
            message_id: sea_orm::ActiveValue::Set(message_id),
            url: sea_orm::ActiveValue::Set(attachment),
            ..Default::default()
        }
    )
        .exec(db).await
        .map(|v| v.last_insert_id)
}

async fn from_attachments(db: &DatabaseConnection, attachments: Vec<String>, message_id: IDType) -> error::MultiResult<IDType, DbErr> {
    let mut results = error::MultiResult::new();
    for attachment in attachments {
        results.push(from_attachment(db, attachment, message_id).await);
    }
    results
}

async fn from_message(db: &DatabaseConnection, message: archive::ArchiveMessage, channel: &model::discord::channel::Model) -> error::MessageResult<Option<IDType>> {
    let db_message_id: IDType = message.id.try_into().map_err(|_| error::MessageError::BadID(message.id))?;
    if let Some(_) = model::discord::Message::find_by_id(db_message_id).one(db).await.map_err(error::MessageError::SeaORM)? {
        return Ok(None)
    };
    let db_user_id: IDType = message.user_id.try_into().map_err(|_| error::MessageError::BadUserID(message.user_id))?;
    let db_in_reply_to: Option<IDType> = message.in_reply_to.map(TryInto::try_into).transpose().map_err(|_| error::MessageError::BadReplyID(message.in_reply_to.unwrap()))?;
    let res = model::discord::Message::insert(
        model::discord::message::ActiveModel {
            id: ActiveValue::Set(db_message_id),
            channel_id: ActiveValue::Set(channel.id),
            user_id: ActiveValue::Set(db_user_id),
            content: ActiveValue::Set(message.content),
            in_reply_to: ActiveValue::Set(db_in_reply_to),
            last_modified: ActiveValue::Set(message.timestamp),
        }
    )
        .exec(db).await
        .map(|v| v.last_insert_id)
        .map_err(error::MessageError::SeaORM)?;
    Ok(Some(res))
}

async fn from_messages(db: &DatabaseConnection, messages: Vec<archive::ArchiveMessage>, channel: &model::discord::channel::Model) -> error::MessagesResult<Option<IDType>> {
    let mut results = error::MultiResult::new();
    for message in messages {
        results.push(from_message(db, message, channel).await);
    }
    results
}
#[derive(Debug)]
pub struct ChannelInfo {
    pub id: IDType,
    pub users: error::UsersResult<Option<IDType>>,
    pub messages: error::MessagesResult<Option<IDType>>,
}

async fn from_channel(db: &DatabaseConnection, channel: archive::ArchiveChannel) -> error::ChannelResult<Option<ChannelInfo>> {
    let db_channel_id: IDType = channel.id.try_into().map_err(|_|error::ChannelError::BadID(channel.id))?;
    match model::discord::Channel::find_by_id(db_channel_id).one(db).await {
        Ok(Some(v)) => return Ok(None),
        Err(e) => return Err(error::ChannelError::SeaORM(e)),
        _ => ()
    }
    let res = model::discord::Channel::insert(
        model::discord::channel::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_channel_id),
            name: sea_orm::ActiveValue::Set(channel.name),
        }
    )
        .exec(db).await
        .map(|v| v.last_insert_id)
        .map_err(error::ChannelError::SeaORM)?;
    let db_channel = model::discord::Channel::find_by_id(db_channel_id)
        .one(db).await
        .map_err(error::ChannelError::SeaORM)?
        .ok_or(error::ChannelError::NotFoundAfterInsert)?;
    Ok(
        Some(
            ChannelInfo{
                id: db_channel.id,
                users: from_users(db, channel.users).await,
                messages: from_messages(db, channel.messages, &db_channel).await,
            }
        )
    )
}

async fn from_archive_path(db: &DatabaseConnection, path: PathBuf) -> Result<Option<ChannelInfo>, error::FileError> {
    if !path.is_file() {
        return Err(error::FileError::NotFound(path));
    }
    let file = File::open(path).map_err(error::FileError::Io)?;
    let reader = BufReader::new(file);
    let archive_channel: archive::ArchiveChannel = serde_json::from_reader(reader).map_err(error::FileError::Serde)?;
    Ok(from_channel(db, archive_channel).await)
}

async fn migration_archive(db: &DatabaseConnection) -> Result<error::MultiResult<Option<ChannelInfo>, error::FileError>, error::FileError> {
    let archive_files = read_dir(ARCHIVE_PATH)
        .map_err(error::FileError::Io)?
        .filter_map(std::result::Result::ok)
        .filter(|item| item.file_type()
            .ok()
            .as_ref()
            .map(std::fs::FileType::is_file)
            .unwrap_or(false)
        )
        .filter(|item| item.path().extension() == Some(OsStr::new("json")))
        .map(|item| item.path());
    let mut results = error::MultiResult::new();
    for archive in archive_files {
        results.push(from_archive_path(db, archive).await);
    }
    Ok(results)
}

async fn ticket_data(db: &DatabaseConnection) -> Result<()> {
    todo!("add ticket data");
}