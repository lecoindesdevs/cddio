use sea_orm::entity::prelude::*;
use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "discord_user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: IDType,
    pub name: String,
    pub avatar: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel 
{}