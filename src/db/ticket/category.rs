use sea_orm::entity::prelude::*;

use crate::db::IDType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cdd_tickets_category")]
pub struct Model {
    /// Identifiant dans la base de données
    #[sea_orm(primary_key)]
    pub id: IDType,
    /// Nom de la catégorie
    name: String, 
    /// Préfix de ticket
    /// 
    /// Le préfix est utilisé pour créer le titre d'un ticket tel que 
    /// `<prefix>_<username>`
    prefix: String,
    /// Description de la catégorie
    description: Option<String>,
    hidden: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation 
{
    #[sea_orm(has_many = "super::Entity")]
    Tickets
}

impl ActiveModelBehavior for ActiveModel 
{}