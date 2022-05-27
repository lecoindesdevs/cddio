//! Le composant help permet d'afficher une aide en fonction de la commande.
//! Il se repose sur le groupe de commande retournée par la fonction [`Component::group_parser`].

use cmp2::declarative::IterType;
use opencdd_components::{self as cmp2, ApplicationCommandEmbed, message};
use opencdd_macros::commands;
use serenity::{async_trait, utils::Colour, client::Context, model::{interactions::{application_command::ApplicationCommandInteraction, InteractionApplicationCommandCallbackDataFlags}, event::InteractionCreateEvent}, builder::CreateEmbed};

use crate::component_system::{self as cmp, command_parser::{self as cmd, Named}, manager::{ArcManager}};

use super::utils::commands::*;

pub struct Help {
    container: cmp2::container::RefContainer,
}

#[derive(Debug, Default)]
struct HelpInfo {
    /// Nom du groupe ou de la commande
    name: String,
    /// Description du groupe ou de la commande
    desc: Option<String>,
    /// Permission (role) requise pour être utilisé
    permission: Option<String>,
    /// Si l'aide concerne un groupe, la liste des sous-groupes, s'il y en a
    groups: Option<Vec<(String, Option<String>)>>,
    /// Si l'aide concerne un groupe, la liste des sous-commande, s'il y en a
    commands: Option<Vec<(String, Option<String>)>>,
    /// Si l'aide concerne une commande, la liste des paramètres, s'il y en a
    params: Option<Vec<(String, Option<String>)>>,
    /// Si aide générale, la liste des composants
    components: Option<Vec<String>>
}

impl Help {
    pub fn new(container: cmp2::container::RefContainer) -> Self {
        Self {
            container
        }
    }
}

#[commands]
impl Help {
    #[command(description="Affiche l'aide d'une commanded ou du bot")]
    async fn help(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>, 
        #[argument(description="Nom de la commande ou du groupe")]
        commande: String
    ) {
        let command_info = self.get_command_info(&commande).await;
        app_cmd.direct_response(ctx, message::error("En cours d'implémentation...")).await;
    }
    #[command(description="Affiche la liste des commandes du bot")]
    async fn liste_commandes(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        let container = self.container.read().await;
        let msg = container.as_ref().iter()
            .filter_map(|comp| comp.declarative())
            .flat_map(|node| node.iter_flat())
            .filter_map(|(fullname, iter_type)| {
                match iter_type {
                    IterType::Command(cmd) => Some((fullname, cmd)),
                    _ => None
                }
            })
            .map(|(fullname, iter_type)| format!("**{}**: {}", fullname, iter_type.description))
            .collect::<Vec<_>>()
            .join("\n");
        app_cmd.direct_response(ctx, message::success(msg)).await;
    }

    async fn get_command_info(&self, command: &str) -> Option<(String, IterType)> {
        let container = self.container.read().await;
        container.as_ref().iter()
            .filter_map(|comp| comp.declarative())
            .flat_map(|node| node.iter_flat())
            .find(|(fullname, _)| fullname == command)
    }
}
