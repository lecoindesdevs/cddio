use opencdd_macros::*;
use serenity::{model::event::Event, client::Context};
use serenity::model::id::RoleId;

use super::utils::app_command::ApplicationCommandEmbed;

trait ComponentEvent {
    fn event(&mut self, ctx: &Context, event: &Event);
}
trait ComponentDeclarative {
    fn declarative(&self) -> &'static [Command];
}

struct Node {
    name: &'static str,
    commands: &'static [Command],
    children: &'static [Node],
}
struct Command {
    name: &'static str,
    description: &'static str,
    args: &'static [Argument],
}
struct Argument {
    name: &'static str,
    type_: serenity :: model :: interactions :: application_command :: ApplicationCommandOptionType,
    description: &'static str,
    optional: bool,
}


struct Test;

#[commands]
#[group(name="tickets", description="Gestion des tickets")]
#[group(name="ticket", description="Commandes dans un ticket")]
#[group(parent="ticket", name="member", description="Gestion des membres dans un ticket")]
impl Test {
    #[command(group="tickets", name="create", description="Créer un ticket")]
    fn tickets_create(&self, 
        ctx: &Context, 
        appcmd: &ApplicationCommandEmbed, 
        #[argument(description="...")]
        categorie: RoleId, 
        #[argument(description="...")]
        pour_qui: User
    ) {} 
    #[command(group="ticket", name="add", description="Ajouter un membre au ticket")]
    fn ticket_member_add(&self, 
        ctx: &Context, 
        appcmd: &ApplicationCommandEmbed, 
        #[argument(description="Qui ajouter au ticket")]
        ajouter_qui: User
    ) {}
    
    // #[event(MessageCreate)]
    // fn test2(&self) {
    //     println!("test2");
    // }
}
