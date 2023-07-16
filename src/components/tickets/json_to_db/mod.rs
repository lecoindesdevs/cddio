mod archive;
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
use serde::Deserialize;
use crate::{
    db::{
        model,
        controller as ctrl, 
        IDType,
    },
    log_warn, log_error, log_info
};


pub enum Error {
    SeaORM(sea_orm::DbErr),
    Io(std::io::Error),
    JSON(serde_json::Error),
    Custom(String)
}

impl From<ctrl::Error> for Error {
    fn from(e: ctrl::Error) -> Self {
        match e {
            ctrl::Error::SeaORM(e) => Error::SeaORM(e),
            ctrl::Error::Custom(e) => Error::Custom(e),
            ctrl::Error::Serenity(e) => unreachable!("Serenity error: {}", e),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

struct Migration;

const TICKET_PATH: &str = "./data/tickets";
const ARCHIVE_PATH: &str = "./data/tickets/archives";

async fn from_category(db: &DatabaseConnection, category: archive::Category) -> bool {
    let discord_category_id: IDType = match category.id.try_into() {
        Ok(v) => v,
        Err(e) => {
            log_error!("Erreur de conversion de l'ID de la catégorie: {}", e);
            return false;
        },
    };
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
        Ok(v) => {
            log_info!("Catégorie {} ajouté", v.last_insert_id);
            true
        },
        Err(e) => {
            log_error!("Erreur lors de l'ajout de la catégorie: {}", e);
            false
        }
    }
}


async fn from_categories(db: &DatabaseConnection, categories: Vec<archive::Category>) -> bool {
    let mut ok = true;
    for category in categories {
        ok &= from_category(db, category).await;
    }
    ok
}

async fn from_user(db: &DatabaseConnection, user: archive::ArchiveUser) -> bool {
    let db_user_id: IDType = match user.id.try_into() {
        Ok(v) => v,
        Err(e) => {
            log_error!("Erreur de conversion de l'ID de l'utilisateur: {}", e);
            return false;
        }
    };
    match model::discord::User::find_by_id(db_user_id).one(db).await {
        Ok(None) => return true,
        Err(e) => {
            log_error!("Erreur lors de l'ajout de l'utilisateur: {}", e);
            return false;
        },
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
        Ok(v) => {
            log_info!("Utilisateur {} ajouté", v.last_insert_id);
            true
        },
        Err(e) => {
            log_error!("Erreur lors de l'ajout de l'utilisateur: {}", e);
            false
        }
    }
}
async fn from_users(db: &DatabaseConnection, users: Vec<archive::ArchiveUser>) -> bool {
    let mut ok = true;
    for user in users {
        ok &= from_user(db, user).await;
    }
    ok
}

async fn from_attachment(db: &DatabaseConnection, attachment: String, message_id: IDType) -> bool {
    let res = model::discord::Attachment::insert(
        model::discord::attachment::ActiveModel {
            message_id: sea_orm::ActiveValue::Set(message_id),
            url: sea_orm::ActiveValue::Set(attachment),
            ..Default::default()
        }
    ).exec(db).await;
    match res {
        Ok(v) => {
            log_info!("Pièce jointe \"{}\" ajouté", v.last_insert_id);
            true
        },
        Err(e) => {
            log_error!("Erreur lors de l'ajout de la pièce jointe: {}", e);
            false
        }
    }
}

async fn from_attachments(db: &DatabaseConnection, attachments: Vec<String>, message_id: IDType) -> bool {
    let mut ok = true;
    for attachment in attachments {
        ok &= from_attachment(db, attachment, message_id).await;
    }
    ok
}

async fn from_message(db: &DatabaseConnection, message: archive::ArchiveMessage, channel: &model::discord::channel::Model) -> bool {
    let db_message_id: IDType = match message.id.try_into() {
        Ok(v) => v,
        Err(e) => {
            log_error!("Dans le salon {} (ID: {}) : Erreur de conversion de l'ID du message {}: {}", channel.name, channel.id, message.id, e);
            return false;
        }
    };
    let db_user_id: IDType = match message.user_id.try_into() {
        Ok(v) => v,
        Err(e) => {
            log_error!("Dans le salon {} (ID: {}) : Dans le message {} : Erreur de conversion de l'ID de l'utilisateur {}: {}", channel.name, channel.id, db_message_id, message.user_id, e);
            return false;
        }
    };
    let db_in_reply_to: Option<IDType> = match message.in_reply_to.map(TryInto::try_into) {
        None => None,
        Some(Ok(v)) => Some(v),
        Some(Err(e)) => {
            log_error!("Dans le salon {} (ID: {}) : Dans le message {} : Erreur de conversion de l'ID de l'utilisateur de réponse {}: {}", channel.name, channel.id, message.id, message.in_reply_to.unwrap(), e);
            return false;
        }
    };
    match model::discord::Message::find_by_id(db_message_id).one(db).await {
        Ok(Some(v)) => {
            log_warn!("Dans le salon {} (ID: {}) : Message {} déjà existant", channel.name, channel.id, v.id);
            return true;
        },
        Err(e) => {
            log_error!("Erreur lors de l'ajout du message: {}", e);
            return false;
        },
        _ => ()
    }
    let res = model::discord::Message::insert(
        model::discord::message::ActiveModel {
            id: ActiveValue::Set(db_message_id),
            channel_id: ActiveValue::Set(channel.id),
            user_id: ActiveValue::Set(db_user_id),
            content: ActiveValue::Set(message.content),
            in_reply_to: ActiveValue::Set(db_in_reply_to),
            last_modified: ActiveValue::Set(message.timestamp),
        }
    ).exec(db).await;
    match res {
        Ok(v) => {
            log_info!("Message {} ajouté", v.last_insert_id);
            true
        },
        Err(e) => {
            log_error!("Erreur lors de l'ajout du message: {}", e);
            false
        }
    }
}

async fn from_messages(db: &DatabaseConnection, messages: Vec<archive::ArchiveMessage>, channel: &model::discord::channel::Model) -> bool {
    let mut ok = true;
    for message in messages {
        ok &= from_message(db, message, channel).await;
    }
    ok
}

async fn from_channel(db: &DatabaseConnection, channel: archive::ArchiveChannel) -> bool {
    let db_channel_id: IDType = match channel.id.try_into() {
        Ok(v) => v,
        Err(e) => {
            log_error!("Erreur de conversion de l'ID du channel: {}", e);
            return false;
        }
    };
    match model::discord::Channel::find_by_id(db_channel_id).one(db).await {
        Ok(Some(v)) => {
            log_warn!("Le channel \"{}\" (ID: {}) existe déjà", v.name, db_channel_id);
            return true
        },
        Err(e) => {
            log_error!("Erreur lors de la recherche du salon {}: {}", db_channel_id, e);
            return false;
        }
        _ => ()
    }
    let res = model::discord::Channel::insert(
        model::discord::channel::ActiveModel {
            id: sea_orm::ActiveValue::Set(db_channel_id),
            name: sea_orm::ActiveValue::Set(channel.name),
        }
    ).exec(db).await;
    match res {
        Ok(v) => {
            log_info!("Channel {} ajouté", v.last_insert_id);
        }
        Err(e) => {
            log_error!("Erreur lors de l'ajout du channel: {}", e);
            return false;
        }
    }
    let db_channel = match model::discord::Channel::find_by_id(db_channel_id).one(db).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            log_error!("(NE DEVRAIT PAS ARRIVER)Le channel (ID: {}) n'existe pas", db_channel_id);
            return false;
        }
        Err(e) => {
            log_error!("(NE DEVRAIT PAS ARRIVER)Erreur lors de la recherche du salon {}: {}", db_channel_id, e);
            return false;
        }
    };
    from_users(db, channel.users).await;
    from_messages(db, channel.messages, &db_channel).await;
    true
}
}