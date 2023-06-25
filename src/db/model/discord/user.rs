use sea_orm::entity::prelude::*;

use crate::db::{
    IDType,
    model::{
        archive,
        ticket
    }
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: IDType,
    pub name: String,
    pub avatar: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "ticket::Entity")]
    OpenedTickets,
}

impl Related<ticket::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OpenedTickets.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}

impl Model {
    pub fn opened_archives(&self) -> Select<ticket::Entity> {
        self.find_related(ticket::Entity)
    }
    pub fn closed_archives(&self) -> Select<ticket::Entity> {
        self.find_linked(archive::ClosedByUser)
    }
}