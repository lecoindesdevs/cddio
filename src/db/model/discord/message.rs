use sea_orm::entity::prelude::*;
use super::{user, attachment, channel};
use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: IDType,
    pub channel_id: IDType,
    pub user_id: IDType,
    pub content: String,
    pub in_reply_to: Option<IDType>,
    pub timestamp: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::InReplyTo",
        to = "user::Column::Id"
    )]
    InReplyTo,
    #[sea_orm(has_many = "attachment::Entity")]
    Attachments,
    #[sea_orm(
        belongs_to = "channel::Entity",
        from = "Column::ChannelId",
        to = "channel::Column::Id"
    )]
    Channel
}

impl Related<attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachments.def()
    }
}

impl Related<channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channel.def()
    }
}

impl ActiveModelBehavior for ActiveModel 
{}

impl Model {
    pub fn attachments(&self) -> Select<attachment::Entity> {
        self.find_related(attachment::Entity)
    }
}