use sea_orm::entity::prelude::*;
use crate::db::{
    IDType,
    discord::{
        user,
        channel
    },
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cdd_archive")]
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
    ClosedBy,
    #[sea_orm(
        belongs_to = "channel::Entity",
        from = "Column::ChannelId",
        to = "channel::Column::Id"
    )]
    Channel
}

macro_rules! def_channels_from_user {
    ($name:ident, $relation:expr) => {
        #[derive(Debug)]
        pub struct $name;

        impl Linked for $name {
            type FromEntity = user::Entity;

            type ToEntity = channel::Entity;

            fn link(&self) -> Vec<RelationDef> {
                vec![
                    $relation.def().rev(),
                    Relation::Channel.def(),
                ]
            }
        }
    };
}

def_channels_from_user!(ChannelOpenedByUser, Relation::OpenedBy);
def_channels_from_user!(ChannelClosedByUser, Relation::ClosedBy);

impl ActiveModelBehavior for ActiveModel 
{}

pub use Entity as Archive;
