pub mod ticket;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    SeaORM(sea_orm::DbErr),
    Custom(String),
}