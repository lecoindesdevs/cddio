//! Le composant help permet d'afficher une aide en fonction de la commande.
//! Il se repose sur le groupe de commande retournée par la fonction [`Component::group_parser`].

use cmp2::declarative::IterType;
use cddio_core::{self as cmp2, ApplicationCommandEmbed, message, message::ToMessage};
use cddio_macros::commands;
use serenity::client::Context;

pub struct Help {
    container: cmp2::container::RefContainer,
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
        let info = self.get_command_info(commande.as_str()).await;
        let msg = match info {
            Some((_, IterType::Command(comm))) => comm.to_message(),
            Some((_, IterType::Node(node))) => node.to_message(),
            None => message::error("Commande inconnue"),
        };
        match app_cmd.direct_response(ctx, msg).await {
            Err(e) => {
                println!("{}", e);
            }
            _ => (),
        }
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
        match app_cmd.direct_response(ctx, message::success(msg)).await {
            Err(e) => {
                println!("{}", e);
            }
            _ => (),
        }
    }

    async fn get_command_info(&self, command: &str) -> Option<(String, IterType)> {
        let container = self.container.read().await;
        container.as_ref().iter()
            .filter_map(|comp| comp.declarative())
            .flat_map(|node| node.iter_flat())
            .find(|(fullname, _)| fullname == command)
    }
}
