use crate::db::controller::discord;
use crate::db::{
    model, 
    controller::Error, 
    IDType
};
use crate::log_info;
use sea_orm::{entity::*, prelude::*};


pub async fn create_ticket(
    ctx: &serenity::client::Context,
    db: &sea_orm::DbConn, 
    category: model::ticket::category::Model, 
    channel_id: serenity::model::id::ChannelId, 
    opened_by: serenity::model::id::UserId
) -> Result<IDType, Error> {
    log_info!("Creating ticket");
    discord::create_channel_if_not_exists(db, ctx, channel_id).await?;
    discord::save_user_from_id(db, ctx, opened_by).await?;
    
    let active_model = model::ticket::ActiveModel {
        category_id: sea_orm::ActiveValue::Set(category.id),
        channel_id: sea_orm::ActiveValue::Set(channel_id.0 as IDType),
        opened_by: sea_orm::ActiveValue::Set(opened_by.0 as IDType),
    };
    let res = model::ticket::Ticket::insert(active_model).exec(db).await.map_err(Error::SeaORM)?;
    log_info!("Channel {} saved", res.last_insert_id);

    Ok(res.last_insert_id)
}
pub async fn is_ticket_exists(db: &sea_orm::DbConn, channel_id: serenity::model::id::ChannelId) -> Result<bool, Error> {
    Ok(model::ticket::Ticket::find_by_id(channel_id.0 as IDType).count(db).await.map_err(Error::SeaORM)? > 0)
}

pub async fn create_ticket_if_not_exist(
    ctx: &serenity::client::Context,
    db: &sea_orm::DbConn, 
    category: model::ticket::category::Model, 
    channel_id: serenity::model::id::ChannelId, 
    opened_by: serenity::model::id::UserId
) -> Result<IDType, Error> {
    if let None = model::ticket::Ticket::find_by_id(channel_id.0 as IDType).one(db).await.map_err(Error::SeaORM)? {
        create_ticket(ctx, db, category, channel_id, opened_by).await?;
    }
    Ok(channel_id.0 as IDType)
}

pub async fn archive_ticket(
    db: &sea_orm::DbConn, 
    ctx: &serenity::client::Context, 
    channel_id: serenity::model::id::ChannelId, 
    closed_by_by: serenity::model::id::UserId
) -> Result<IDType, Error> {
    log_info!("Archiving ticket");
    match model::ticket::Ticket::find_by_id(channel_id.0 as IDType).one(db).await {
        Ok(None) => Err(Error::Custom("Ticket not found".to_string()))?,
        Err(e) => Err(Error::SeaORM(e))?,
        _ => ()
    }
    discord::save_user_from_id(db, ctx, closed_by_by).await?;
    super::discord::save_channel(db, ctx, channel_id).await?;
    let active_model = model::archive::ActiveModel {
        ticket_id: sea_orm::ActiveValue::Set(channel_id.0 as IDType),
        closed_by: sea_orm::ActiveValue::Set(closed_by_by.0 as IDType),
        ..Default::default()
    };
    let res = model::archive::Archive::insert(active_model).exec(db).await.map_err(Error::SeaORM)?;
    log_info!("Ticket {} archived", res.last_insert_id);
    Ok(res.last_insert_id)
}

pub async fn add_category(
    db: &sea_orm::DbConn, 
    name: String, 
    prefix: String, 
    discord_category_id: serenity::model::id::ChannelId,
    description: Option<String>, 
    hidden: Option<bool>
) -> Result<IDType, Error> {
    log_info!("Adding category");
    let discord_category_id = discord_category_id.0
        .try_into()
        .map_err(|e| Error::Custom(format!("Unable to convert ID from u64 to i64: {:?}", e)))?;
    let active_model = model::ticket::category::ActiveModel {
        name: sea_orm::ActiveValue::Set(name),
        prefix: sea_orm::ActiveValue::Set(prefix),
        discord_category_id: sea_orm::ActiveValue::Set(discord_category_id),
        description: sea_orm::ActiveValue::Set(description),
        hidden: sea_orm::ActiveValue::Set(hidden.unwrap_or(false)),
        .. Default::default()
    };
    let res = model::ticket::Category::insert(active_model).exec(db).await.map_err(Error::SeaORM)?;
    log_info!("Category {} saved", res.last_insert_id);
    Ok(res.last_insert_id)
}

pub async fn remove_category(db: &sea_orm::DbConn, category_id: IDType) -> Result<(), Error> {
    log_info!("Removing category {}", category_id);
    model::ticket::Category::delete_by_id(category_id).exec(db).await.map_err(Error::SeaORM)?;
    log_info!("Category {} removed", category_id);
    Ok(())
}