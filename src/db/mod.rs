pub mod discord;

use sea_orm::{Database, DbConn, DbErr, Schema, ConnectionTrait, TransactionTrait};

pub async fn start_db(url: &str) -> Result<DbConn, DbErr> {
    let db = Database::connect(url).await?;
    check_tables(&db).await?;
    Ok(db)
}

async fn check_tables(db: &DbConn) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let transaction = db.begin().await?;

    transaction.execute(builder.build(schema.create_table_from_entity(discord::User).if_not_exists())).await?;
    transaction.execute(builder.build(schema.create_table_from_entity(discord::Channel).if_not_exists())).await?;
    transaction.execute(builder.build(schema.create_table_from_entity(discord::Message).if_not_exists())).await?;
    transaction.execute(builder.build(schema.create_table_from_entity(discord::Attachment).if_not_exists())).await?;
    transaction.commit().await?;
    
    Ok(())
}