use crate::db::{
    model, 
    controller::Error, 
    IDType
};
use crate::log_info;
use sea_orm::{entity::*, TransactionTrait};

pub async fn create_ticket(db: &sea_orm::DbConn, category: model::ticket::category::Model, channel_id: serenity::model::id::ChannelId, opened_by: serenity::model::id::UserId) -> Result<IDType, Error> {
    log_info!("Creating ticket");
    let txn = db.begin().await.map_err(Error::SeaORM)?;
    let active_model = model::ticket::ActiveModel {
        category_id: sea_orm::ActiveValue::Set(category.id),
        channel_id: sea_orm::ActiveValue::Set(channel_id.0 as IDType),
        opened_by: sea_orm::ActiveValue::Set(opened_by.0 as IDType),
    };
    let res = model::ticket::Ticket::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
    log_info!("Channel {} saved", res.last_insert_id);

    Ok(res.last_insert_id)
}

pub async fn archive_ticket(db: &sea_orm::DbConn, ctx: &serenity::client::Context, channel_id: serenity::model::id::ChannelId, closed_by_by: serenity::model::id::UserId) -> Result<IDType, Error> {
    use model::ticket::category::Entity as Category;
    log_info!("Archiving ticket");
    let txn = db.begin().await.map_err(Error::SeaORM)?;
    // Create ticket if it doesn't exist. It happens if the ticket is not created by this bot.
    if let None = model::ticket::Ticket::find_by_id(channel_id.0 as IDType).one(&txn).await.map_err(Error::SeaORM)? {
        let bot_id = ctx.cache.current_user().id;
        let category = Category::find()
            .one(db).await
            .map_err(Error::SeaORM)?
            .map_or_else(
                || Err(Error::Custom("No categories".to_string())), 
                |c| Ok(c)
            )?;
        create_ticket(db, category, channel_id, bot_id).await?;
    }
    super::discord::save_channel(db, ctx, channel_id).await?;
    let active_model = model::archive::ActiveModel {
        ticket_id: sea_orm::ActiveValue::Set(channel_id.0 as IDType),
        closed_by: sea_orm::ActiveValue::Set(closed_by_by.0 as IDType),
        ..Default::default()
    };
    let res = model::archive::Archive::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
    log_info!("Ticket {} archived", res.last_insert_id);
    Ok(res.last_insert_id)
}

pub async fn add_category(db: &sea_orm::DbConn, name: String, prefix: String, description: Option<String>, hidden: Option<bool>) -> Result<IDType, Error> {
    log_info!("Adding category");
    let txn = db.begin().await.map_err(Error::SeaORM)?;
    let active_model = model::ticket::category::ActiveModel {
        name: sea_orm::ActiveValue::Set(name),
        prefix: sea_orm::ActiveValue::Set(prefix),
        description: sea_orm::ActiveValue::Set(description),
        hidden: sea_orm::ActiveValue::Set(hidden.unwrap_or(false)),
        .. Default::default()
    };
    let res = model::ticket::Category::insert(active_model).exec(&txn).await.map_err(Error::SeaORM)?;
    log_info!("Category {} saved", res.last_insert_id);
    Ok(res.last_insert_id)
}

pub async fn remove_category(db: &sea_orm::DbConn, category_id: IDType) -> Result<(), Error> {
    log_info!("Removing category {}", category_id);
    model::ticket::Category::delete_by_id(category_id).exec(db).await.map_err(Error::SeaORM)?;
    log_info!("Category {} removed", category_id);
    Ok(())
}