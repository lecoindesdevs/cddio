use sea_orm::entity::prelude::*;
use super::message;
use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_attachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: IDType,
    pub message_id: IDType,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "message::Entity",
        from = "Column::MessageId",
        to = "message::Column::Id"
    )]
    Message
}

impl Related<message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}