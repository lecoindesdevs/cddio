pub mod ticket;
pub mod discord;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    SeaORM(sea_orm::DbErr),
    Custom(String),
}