use sea_orm::DatabaseConnection;
use serde::Deserialize;
use crate::db::{
    model,
    controller as ctrl,
};

#[derive(Deserialize, Debug, Clone)]
struct Category {
    name: String,
    prefix: String,
    id: u64,
    desc: Option<String>,
    tickets: Vec<String>,
    #[serde(default)]
    hidden: bool,
}
mod archive;

struct Migration;

const ARCHIVE_PATH: &str = "./data/tickets/archives";

async fn categories(db: &DatabaseConnection) -> Result<(), Error> {
    let categories = Category::find()
        .all(db)
        .await
        .map_err(Error::SeaOrm)?;
    Ok(categories)
}