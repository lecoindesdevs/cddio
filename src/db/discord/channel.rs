use sea_orm::entity::prelude::*;
use super::{user, message};
use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_channel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: IDType,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "message::Entity")]
    Messages,
}

impl Related<message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}