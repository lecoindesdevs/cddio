use sea_orm::entity::prelude::*;
use super::{user, attachment, channel};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u64,
    pub channel_id: u64,
    pub user_id: u64,
    pub content: String,
    pub in_reply_to: Option<u64>,
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