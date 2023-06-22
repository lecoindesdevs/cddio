use sea_orm::{
    entity::prelude::*,
    Select
};
use super::message;
use crate::db::{
    IDType,
    ticket
};

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
    #[sea_orm(has_one = "ticket::Entity")]
    Ticket,
}

impl Related<message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}

impl Model {
    pub fn messages(&self) -> Select<message::Entity> {
        use sea_orm::QueryOrder;
        self
            .find_related(message::Entity)
            .order_by_asc(message::Column::Id)
    }
}