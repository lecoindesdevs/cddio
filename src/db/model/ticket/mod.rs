pub mod category;

pub use category::Entity as Category;
pub use Entity as Ticket;


use sea_orm::entity::prelude::*;
use crate::db::{
    IDType,
    model::discord
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cdd_ticket")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub channel_id: IDType,

    pub category_id: IDType,
    pub opened_by: IDType,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "category::Entity",
        from = "Column::CategoryId",
        to = "category::Column::Id"
    )]
    Category,
    #[sea_orm(
        belongs_to = "discord::channel::Entity",
        from = "Column::ChannelId",
        to = "discord::channel::Column::Id"
    )]
    Channel,
    #[sea_orm(
        belongs_to = "discord::user::Entity",
        from = "Column::OpenedBy",
        to = "discord::user::Column::Id"
    )]
    OpenedBy,
}

impl Related<discord::channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channel.def()
    }
}
impl Related<category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}
impl Related<discord::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OpenedBy.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}