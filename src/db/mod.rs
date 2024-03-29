pub mod model;
pub mod controller;

use crate::{log_info};

pub type IDType = i64;

use sea_orm::{Database, DbConn, DbErr, Schema, ConnectionTrait, TransactionTrait};

pub async fn start_db(url: &str) -> Result<DbConn, DbErr> {
    let db = Database::connect(url).await?;
    check_tables(&db).await?;
    Ok(db)
}

macro_rules! create_tables_if_not_exists {
    ($transaction:ident, $schema:ident, $builder:ident $(, $name:path)*) => {
        $(
            log_info!("Creating table {}", stringify!($name));
            let res = $transaction.execute(
                $builder.build($schema.create_table_from_entity($name).if_not_exists())
            ).await?;
            log_info!("Table {} created: {:?}", stringify!($name), res);
        )*
    };
}

async fn check_tables(db: &DbConn) -> Result<(), DbErr> {
    use model::*;
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let transaction = db.begin().await?;

    log_info!("Creating tables");

    create_tables_if_not_exists!(transaction, schema, builder, 
        archive::Archive,
        discord::Attachment,
        discord::Channel, 
        discord::Message, 
        discord::User, 
        ticket::Category,
        ticket::Ticket
    );
    match transaction.commit().await {
        Ok(_) => {
            log_info!("Tables created");
            Ok(())
        },
        e => e,
    }
}