use std::fmt::Display;

pub mod ticket;
pub mod discord;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    SeaORM(sea_orm::DbErr),
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Serenity(e) => {
                f.write_str("Serenity error: ")?;
                e.fmt(f)
            },
            Error::SeaORM(e) => {
                f.write_str("sea-orm error: ")?;
                e.fmt(f)
            }
            Error::Custom(e) => {
                f.write_str("Database error: ")?;
                e.fmt(f)
            }
        }
        
    }
}