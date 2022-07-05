use std::path::PathBuf;
use futures::TryFutureExt;
use log::*;
use futures_locks::RwLock;
use opencdd_components::{message, ApplicationCommandEmbed};
use opencdd_macros::commands;
use serde::{Serialize, Deserialize};
use serenity::{
    client::Context,
    model::{id::*, channel::Message},
    model::event::{
        Event::InteractionCreate,
        InteractionCreateEvent,
    },
    model::interactions:: {
        Interaction,
        message_component::MessageComponentInteraction
    }, builder::{CreateMessage, CreateSelectMenuOption}
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

impl From<CategoryTicket> for CreateSelectMenuOption {
    fn from(ticket: CategoryTicket) -> Self {
        let mut menu_option = CreateSelectMenuOption::new(&ticket.name, &ticket.name);
        menu_option
            .description(ticket.desc.unwrap_or_default());
        menu_option
    }
} 
impl From<&CategoryTicket> for CreateSelectMenuOption {
    fn from(ticket: &CategoryTicket) -> Self {
        let mut menu_option = CreateSelectMenuOption::new(&ticket.name, &ticket.name);
        menu_option
            .description(ticket.desc.clone().unwrap_or_default());
        menu_option
    }
} 

impl Tickets {
    /// Créer un nouveau composant de gestion des tickets
    pub fn new() -> Self {
        Self {
            data: RwLock::new(Data::from_file("tickets").unwrap()),
            archives_folder: PathBuf::from("data/archives/tickets")
        }
    }
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
        let mut msg = match app_cmd.0.channel_id.send_message(ctx, |msg| msg.content("Sélectionnez le type de ticket que vous souhaitez créer :")).await {
            Ok(msg) => msg,
            Err(err) => {
                error!("Erreur lors de l'envoi du message: {}", err);
                return;
            }
        };
        self.update_menu(ctx, &mut msg).await.unwrap_or_else(|e| {
            error!("Erreur lors de l'envoi du message: {}", e);
        });
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
        {
            let data = self.data.read().await;
            let data = data.read();
            for category in &data.categories {
                if category.name == name {
                    app_cmd.direct_response(ctx, message::error("Cette catégorie existe déjà")).await.unwrap_or_else(|e| {
                        error!("Erreur lors de l'envoi du message: {}", e);
                    });
                    return;
                }
            }
        }
        {
            let mut data = self.data.write().await;
            let mut data = data.write();
            data.categories.push(CategoryTicket {
                name,
                prefix,
                id: category_id.0,
                desc,
                tickets: vec![]
            });
        }
        {
            let data = self.data.read().await;
            let data = data.read();
            let category = data.categories.last().unwrap();
            let mut msg = message::Message::new();
            let mut embed = message::Embed::default();
            embed.color(message::COLOR_INFO);
            embed.title("Catégorie créée");
            embed.field(&category.name, category.desc.clone().unwrap_or_else(|| "*Aucune desscription*".into()), false);
            msg.add_embed(|e| {*e=embed; e});
            app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
                error!("Erreur lors de l'envoi du message: {}", e);
            });
        }
    }
    #[command(group="categories", name="remove", description="Supprime une catégorie de ticket")]
    async fn remove_categorie(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="nom", description="Nom de la catégorie")]
        name: String
    ) {
        let mut data = self.data.write().await;
        let mut data = data.write();
        let pos = match data.categories.iter().position(|category| category.name == name) {
            Some(pos) => pos,
            None => {
                app_cmd.direct_response(ctx, message::error("Cette catégorie n'existe pas")).await.unwrap_or_else(|e| {
                    error!("Erreur lors de l'envoi du message: {}", e);
                });
                return;
            }
        };
        let msg = {
            let category = &data.categories[pos];
            let mut msg = message::Message::new();
            let mut embed = message::Embed::default();
            embed.color(message::COLOR_INFO);
            embed.title("Catégorie supprimée");
            embed.field(&category.name, category.desc.clone().unwrap_or_else(|| "*Aucune desscription*".into()), false);
            msg.add_embed(|e| {*e=embed; e});
            msg
        };
        data.categories.remove(pos);

        app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
            error!("Erreur lors de l'envoi du message: {}", e);
        });
    }
    #[command(group="categories", name="list", description="Liste les catégories de ticket")]
    async fn list_categories(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        let data = self.data.read().await;
        let data = data.read();
        let mut msg = message::Message::new();
        let mut embed = message::Embed::default();
        embed.title("Liste des catégories");
        embed.color(message::COLOR_INFO);
        for category in &data.categories {
            embed.field(&category.name, category.desc.clone().unwrap_or_else(|| "*Aucune desscription*".into()), false);
            
        }
        msg.add_embed(|e| {*e=embed; e});
        app_cmd.direct_response(ctx, msg).await.unwrap_or_else(|e| {
            error!("Erreur lors de l'envoi du message: {}", e);
        });
    }
    #[command(group="ticket", description="Ajoute une personne au ticket")]
    async fn add_member(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>,
        #[argument(name="qui", description="Personne à ajouter au ticket")]
        person: UserId
    ) {
        todo!()
    }
    #[message_component(custom_id="menu_ticket_create")]
    async fn on_menu_ticket_create(&self, ctx: &Context, msg: &MessageComponentInteraction) {
        msg.create_interaction_response(ctx, |resp|{
            let menus_str = msg.data.values.join(", ");
            resp.interaction_response_data(|inter| inter.content(format!("Vous avez appuyé sur le menu '{}'", menus_str)))
        }).await.unwrap();
    }
    #[message_component(custom_id="button_ticket_close")]
    async fn on_button_ticket_close(&self, ctx: &Context, msg: &MessageComponentInteraction) {
        msg.create_interaction_response(ctx, |resp|{
            let menus_str = msg.data.values.join(", ");
            resp.interaction_response_data(|inter| inter.content(format!("Vous avez appuyé sur le menu '{}'", menus_str)))
        }).await.unwrap();
    }
}

impl Tickets {
    async fn update_menu(&self, ctx: &Context, msg: &mut Message) -> serenity::Result<()>{
        let options = self.data.read().await.read().categories.iter().map(|cat| cat.into()).collect::<Vec<_>>();
        msg.edit(ctx, |msg|{
            msg.components(|comp| {
                comp.create_action_row(|action| {
                    action.create_select_menu(|menu| {
                        menu.options(|opts|{
                            opts.set_options(options)
                        }).custom_id("menu_ticket_create")
                    })
                })
            })
        }).await
    }
}