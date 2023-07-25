use std::fmt::Display;

pub mod ticket;
pub mod discord;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    SeaORM(sea_orm::DbErr),
    BadConversionId(u64),
    File(FileError),
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
            Error::BadConversionId(e) => {
                f.write_str("Bad conversion id: ")?;
                e.fmt(f)
            }
            Error::File(e) => {
                f.write_str("File error: ")?;
                e.fmt(f)
            }
            Error::Custom(e) => {
                f.write_str("Database error: ")?;
                e.fmt(f)
            }
        }
        
    }
}


#[derive(Debug)]
pub enum FileError {
    Serenity(serenity::Error),
    Io(std::io::Error),
}

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Serenity(e) => {
                f.write_str("Serenity error: ")?;
                e.fmt(f)
            },
            FileError::Io(e) => {
                f.write_str("Io error: ")?;
                e.fmt(f)
            }
        }
    }
}