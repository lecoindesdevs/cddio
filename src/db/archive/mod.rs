use sea_orm::entity::prelude::*;
use super::discord::user;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    pub channel_id: u64,
    pub opened_by: u64,
    pub closed_by: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::OpenedBy",
        to = "user::Column::Id"
    )]
    OpenedBy,
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::ClosedBy",
        to = "user::Column::Id"
    )]
    ClosedBy
}

impl ActiveModelBehavior for ActiveModel 
{}

pub use Entity as Archive;
