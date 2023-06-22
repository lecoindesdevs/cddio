use sea_orm::entity::prelude::*;
use crate::db::{
    IDType,
    model::{
        discord::user,
        ticket
    }
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cdd_archive")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: IDType,
    pub ticket_id: IDType,
    pub closed_by: IDType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::ClosedBy",
        to = "user::Column::Id"
    )]
    ClosedBy,
    #[sea_orm(
        belongs_to = "ticket::Entity",
        from = "Column::TicketId",
        to = "ticket::Column::ChannelId"
    )]
    Ticket
}

#[derive(Debug)]
pub struct ClosedByUser;

impl Linked for ClosedByUser {
    type FromEntity = user::Entity;

    type ToEntity = ticket::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::ClosedBy.def().rev(),
            Relation::Ticket.def(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel 
{}

pub use Entity as Archive;
