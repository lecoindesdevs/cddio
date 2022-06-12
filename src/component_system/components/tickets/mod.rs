use std::path::PathBuf;

use futures_locks::RwLock;
use opencdd_components::{ApplicationCommandEmbed};
use opencdd_macros::commands;
use serde::{Serialize, Deserialize};
use serenity::{
    client::Context,
    model::{id::*},
    model::event::{
        Event::InteractionCreate,
        InteractionCreateEvent,
    },
    model::interactions:: {
        Interaction,
        message_component::MessageComponentInteraction
    }
};

use super::utils::Data;

/// Le composant de gestion des tickets
pub struct Tickets {
    /// Données persistantes
    data: RwLock<Data<DataTickets>>,
    /// Dossier de sauvegarde des tickets
    /// 
    /// Dès que les tickets sont supprimés, ils sont enregistrés dans ce dossier.
    archives_folder: PathBuf
}

/// Données persistantes du composant
/// 
/// A chaque écriture dans le fichier de données, le fichier est sauvegardé
#[derive(Serialize, Deserialize, Default, Debug)]
struct DataTickets {
    /// Identifiants du channel et du message pour choisir le type de ticket
    /// Ces identifiants est enregistré pour pouvoir le remplacer si nécessaire
    msg_choose: Option<(u64, u64)>,
    /// [Catégories] de tickets
    /// 
    /// [Catégories]: CategoryTicket
    categories: Vec<CategoryTicket>,
}

/// Catégorie de tickets
#[derive(Serialize, Deserialize, Default, Debug)]
struct CategoryTicket {
    /// Nom de la catégorie
    name: String, 
    /// Préfix de ticket
    /// 
    /// Le préfix est utilisé pour créer le titre d'un ticket tel que 
    /// `<prefix>_<username>`
    prefix: String,
    /// Identifiant de la catégorie Discord
    id: u64,
    /// Description de la catégorie
    desc: Option<String>,
    /// Tickets créés dans cette catégorie
    tickets: Vec<String>,
}

#[commands]
#[group(name="tickets", description="Gestion des tickets")]
#[group(parent="tickets", name="categories", description="Gestion des catégories de tickets")]
#[group(name="ticket", description="Commandes dans un ticket")]
impl Tickets {
    #[command(group="tickets", description="Assigne le salon de création de tickets")]
    async fn set_channel(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="salon", description="Salon textuel")]
        chan: ChannelId
    ) {
        todo!()
    }
    #[command(group="tickets", name="close", description="Ferme le ticket actuel")]
    async fn ticket_close(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        todo!()
    }
    #[command(group="categories", name="add", description="Ajoute une catégorie de ticket. À ne pas confondre avec les catégories discord")]
    async fn add_categorie(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="nom", description="Nom de la catégorie")]
        name: String,
        #[argument(description="Catégorie Discord où les tickets seront créés", name="categorie_discord")]
        category_id: ChannelId,
        #[argument(description="Préfixe des tickets", name="prefix")]
        prefix: String,
        #[argument(description="Description de la catégorie", name="description")]
        desc: Option<String>
    ) {
        todo!()
    }
    #[command(group="categories", name="remove", description="Supprime une catégorie de ticket")]
    async fn remove_categorie(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="nom", description="Nom de la catégorie")]
        name: String
    ) {
        todo!()
    }
    #[command(group="categories", name="list", description="Liste les catégories de ticket")]
    async fn list_categories(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        todo!()
    }
    #[command(group="ticket", description="Ajoute une personne au ticket")]
    async fn add_member(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="qui", description="Personne à ajouter au ticket")]
        person: UserId
    ) {
        todo!()
    }
    #[event(InteractionCreate(InteractionCreateEvent{interaction: Interaction::MessageComponent(message_interaction), ..}))]
    async fn on_msg_component(&self, ctx: &Context, message_interaction: &MessageComponentInteraction) {
        todo!()
    }
}