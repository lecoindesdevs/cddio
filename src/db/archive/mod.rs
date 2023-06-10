use sea_orm::entity::prelude::*;
use super::discord::user;
use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: IDType,
    pub channel_id: IDType,
    pub opened_by: IDType,
    pub closed_by: IDType,
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
