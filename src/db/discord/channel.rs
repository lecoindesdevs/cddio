use sea_orm::entity::prelude::*;
use super::{user, message};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_channel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u64,
    pub name: String,
    pub opened_by: u64,
    pub closed_by: Option<u64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "message::Entity")]
    Messages,
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

impl Related<message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}