use opencdd_macros::*;
use serenity::model::event::{MessageCreateEvent, ReadyEvent};
use serenity::{client::Context};
use serenity::model::id::RoleId;

use opencdd_components::ApplicationCommandEmbed;

pub struct Test;

#[commands]
#[group(name="tickets", description="Gestion des tickets")]
#[group(name="ticket", description="Commandes dans un ticket")]
#[group(parent="ticket", name="member", description="Gestion des membres dans un ticket")]
impl Test {
    #[command(group="tickets", name="create", description="Créer un ticket")]
    async fn tickets_create(&self, 
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>, 
        #[argument(description="...")]
        categorie: RoleId, 
        #[argument(name="who", description="...")]
        pour_qui: serenity::model::user::User
    ) {} 
    #[command(group="member", name="add", description="Ajouter un membre au ticket")]
    async fn ticket_member_add(&self, 
        ctx: &Context, 
        appcmd: ApplicationCommandEmbed<'_>, 
        #[argument(description="Qui ajouter au ticket")]
        ajouter_qui: Option<serenity::model::user::User>
    ) {}
    
    #[event(MessageCreate)]
    async fn test2(&self, ctx: &Context, msg: &MessageCreateEvent) {
        println!("test2");
    }
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
        println!("test démarrage");
    }
}
