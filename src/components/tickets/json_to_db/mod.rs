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
    ActiveValue, QuerySelect
};
use crate::db::{
    model,
    IDType,
};

use self::error::CategoriesResult;


struct Migration;

const DATA_TICKET_PATH: &str = "./data/tickets.json";
const ARCHIVE_PATH: &str = "./data/tickets/archives";

async fn from_category<C: ConnectionTrait>(db: &C, category: archive::Category) -> error::CategoryResult<IDType> {
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


async fn from_categories<C: ConnectionTrait>(db: &C, categories: Vec<archive::Category>) -> error::CategoriesResult<IDType> {
    let mut results = error::MultiResult::new();
    for category in categories {
        results.push(from_category(db, category).await);
    }
    results
}

async fn from_user(db: &DatabaseConnection, user: archive::ArchiveUser) -> error::UserResult<Option<IDType>> {
    let db_user_id: IDType = user.id.try_into().map_err(|_| error::UserError::BadID(user.id))?;
    match model::discord::User::find_by_id(db_user_id).one(db).await {
        Ok(Some(_)) => return Ok(None),
        Err(e) => return Err(error::UserError::SeaORM(e)),
        _ => ()
    }
    let mut username = user.name;
    if username.ends_with("#0") {
        //drop the last 2 characters
        username = username[..username.len()-2].to_string();
    }
    let res = model::discord::User::insert(
        model::discord::user::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_user_id),
            name: sea_orm::ActiveValue::Set(username),
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

async fn from_attachment<C: ConnectionTrait>(db: &C, attachment: String, message_id: IDType) -> Result<IDType, DbErr> {
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

async fn from_attachments<C: ConnectionTrait>(db: &C, attachments: Vec<String>, message_id: IDType) -> error::MultiResult<IDType, DbErr> {
    let mut results = error::MultiResult::new();
    for attachment in attachments {
        results.push(from_attachment(db, attachment, message_id).await);
    }
    results
}
#[derive(Debug)]
pub struct MessageInfo{
    pub id: IDType,
    pub attachments: error::MultiResult<IDType, DbErr>,
}

async fn from_message<C: ConnectionTrait>(db: &C, message: archive::ArchiveMessage, channel: &model::discord::channel::Model) -> error::MessageResult<Option<MessageInfo>> {
    let db_message_id: IDType = message.id.try_into().map_err(|_| error::MessageError::BadID(message.id))?;
    if let Some(_) = model::discord::Message::find_by_id(db_message_id).one(db).await.map_err(error::MessageError::SeaORM)? {
        return Ok(None)
    };
    let db_user_id: IDType = message.user_id.try_into().map_err(|_| error::MessageError::BadUserID(message.user_id))?;
    let db_in_reply_to: Option<IDType> = message.in_reply_to.map(TryInto::try_into).transpose().map_err(|_| error::MessageError::BadReplyID(message.in_reply_to.unwrap()))?;
    model::discord::Message::insert(
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
    Ok(Some(MessageInfo {
        id: db_message_id,
        attachments: from_attachments(db, message.attachments, db_message_id).await
    }))
}

async fn from_messages(db: &DatabaseConnection, messages: Vec<archive::ArchiveMessage>, channel: &model::discord::channel::Model) -> error::MessagesResult<Option<MessageInfo>> {
    use sea_orm::TransactionTrait;
    let mut results = error::MultiResult::new();
    let txn = db.begin().await.expect("Failed to begin transaction");
    for message in messages.into_iter().rev() {
        results.push(from_message(&txn, message, channel).await);
    }
    txn.commit().await.expect("Failed to commit transaction");
    results
}
#[derive(Debug)]
pub struct ChannelInfo {
    pub id: IDType,
    pub users: error::UsersResult<Option<IDType>>,
    pub messages: error::MessagesResult<Option<MessageInfo>>,
}

async fn from_channel(db: &DatabaseConnection, channel: archive::ArchiveChannel) -> error::ChannelResult<Option<ChannelInfo>> {
    let db_channel_id: IDType = channel.id.try_into().map_err(|_|error::ChannelError::BadID(channel.id))?;
    match model::discord::Channel::find_by_id(db_channel_id).one(db).await {
        Ok(Some(_)) => return Ok(None),
        Err(e) => return Err(error::ChannelError::SeaORM(e)),
        _ => ()
    }
    let _res = model::discord::Channel::insert(
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

#[derive(Debug)]
pub struct ArchiveInfo {
    archive: IDType,
    ticket: IDType,
    channel: ChannelInfo,
}

async fn category_from_name(db: &DatabaseConnection, name: &str) -> error::CategoryResult<IDType> {
    if let Some(pos) = name.find(&['_', '-']) {
        let prefix = &name[..pos];
        match model::ticket::Category::find().filter(model::ticket::category::Column::Prefix.eq(prefix)).column(model::ticket::category::Column::Id).one(db).await {
            Ok(Some(v)) => return Ok(v.id),
            Err(e) => return Err(error::CategoryError::SeaORM(e)),
            _ => (),
        }
    }
    model::ticket::Category::find()
        .column(model::ticket::category::Column::Id)
        .one(db).await
        .map_err(error::CategoryError::SeaORM)
        .and_then(|v| {
            v.ok_or(error::CategoryError::NotFound)
        })
        .map(|v| v.id)
        
    
}

async fn from_archive_path(db: &DatabaseConnection, path: PathBuf, default_user_id: IDType) -> Result<Option<ArchiveInfo>, error::ArchiveError> {
    if !path.is_file() {
        return Err(error::ArchiveError::File(error::FileError::NotFound(path)));
    }
    let file = File::open(path).map_err(|e| error::ArchiveError::File(error::FileError::Io(e)))?;
    let reader = BufReader::new(file);
    let archive_channel: archive::ArchiveChannel = serde_json::from_reader(reader).map_err(|e| error::ArchiveError::File(error::FileError::Serde(e)))?;
    let default_user = archive::ArchiveUser {
        id: default_user_id as u64,
        name: "CDDIO".to_string(),
        avatar: "https://cdn.discordapp.com/avatars/871363223801196594/96899439c4a563f81ae19198e7692762?size=1024".to_string(),
    };
    from_user(db, default_user).await.map_err(error::ArchiveError::ClosedBy)?;
    let closed_by_user_id = match &archive_channel.closed_by {
        Some(v) => { 
            let id = v.user.id as IDType;
            from_user(db, v.user.clone()).await.map_err(error::ArchiveError::ClosedBy)?;
            id
        },
        None => default_user_id,
    };
    let category_id = category_from_name(db, &archive_channel.name).await.map_err(error::ArchiveError::Category)?;

    let channel = from_channel(db, archive_channel).await.map_err(error::ArchiveError::Channel)?;
    let channel = match channel {
        Some(v) => v,
        None => return Ok(None),
    };
    

    let ticket = model::ticket::Entity::insert(
        model::ticket::ActiveModel {
            channel_id: sea_orm::ActiveValue::Set(channel.id),
            category_id: sea_orm::ActiveValue::Set(category_id),
            opened_by: sea_orm::ActiveValue::Set(default_user_id),
            ..Default::default()
        }
    )
        .exec(db).await
        .map(|v| v.last_insert_id)
        .map_err(error::ArchiveError::TicketInsert)?;
    
    let res = model::archive::Entity::insert(
        model::archive::ActiveModel {
            ticket_id: sea_orm::ActiveValue::Set(ticket),
            closed_by: sea_orm::ActiveValue::Set(closed_by_user_id),
            ..Default::default()
        }
    )
        .exec(db).await
        .map(|v| ArchiveInfo {
            archive: v.last_insert_id,
            ticket,
            channel,
        })
        .map_err(error::ArchiveError::ArchiveInsert)?;
    Ok(Some(res))
}

async fn migration_archives(db: &DatabaseConnection, default_user_id: IDType) -> error::FileResult<error::ArchivesResult<Option<ArchiveInfo>>> {
    // Get an iterator of all archive files
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
        results.push(from_archive_path(db, archive, default_user_id).await);
    }
    let new_path = std::path::Path::new(ARCHIVE_PATH).parent().unwrap().join("_archives");
    if !cfg!(debug_assertions) {
        std::fs::rename(ARCHIVE_PATH, new_path);
    }
    Ok(results)
}

async fn migration_data_tickets(db: &DatabaseConnection) -> error::FileResult<CategoriesResult<IDType>> {
    use super::{Tickets, MessageChoice};
    use sea_orm::TransactionTrait;
    let file = File::open(DATA_TICKET_PATH).map_err(error::FileError::Io)?;
    let reader = BufReader::new(file);
    let old_data: archive::DataTickets = serde_json::from_reader(reader).map_err(error::FileError::Serde)?;
    {
        let component_data = Tickets::new_data();
        let mut component_data = component_data.write().await;
        if let Some(old_msg_choose) = old_data.msg_choose {
            component_data.message_choice = Some(MessageChoice {
                channel_id: old_msg_choose.0,
                message_id: old_msg_choose.1,
            });
        }
    }
    let categories_result = match db.begin().await {
        Ok(txn) => {
            let mut res = from_categories(&txn, old_data.categories).await;
            if let Err(e) = txn.commit().await.map_err(error::CategoryError::SeaORM) {
                res.push_err(e);
            }
            res
        },
        Err(_) => from_categories(db, old_data.categories).await,
    };
    
    Ok(categories_result)
}

pub async fn do_migration(db: &DatabaseConnection, default_user_id: IDType) -> error::MigrationResult<error::ArchivesResult<Option<ArchiveInfo>>> {
    migration_data_tickets(db).await.map_err(error::MigrationError::DataTickets)?;
    Ok(migration_archives(db, default_user_id).await.map_err(error::MigrationError::Archives)?)
}