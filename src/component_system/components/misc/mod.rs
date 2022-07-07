/// Sur 250 metres, on passe d'une vitesse de 0 m/s a 270 m/s.
/// calculer la durée pour atteindre cette vitesse en secondes
/// https://www.wolframalpha.com/input/?i=distance+between+0+and+270+m%2Fs+in+seconds

use log::*;
use opencdd_components::{self as cmp2, ApplicationCommandEmbed, message, message::ToMessage};
use opencdd_macros::commands;
use serenity::{
    client::Context, 
    model::{id::ApplicationId, gateway::Ready, permissions::Permissions, event::ReadyEvent}
};

pub struct Misc {
    app_id: ApplicationId,
    bot_permissions: u64,
    container: cmp2::container::RefContainer,
}

#[commands]
impl Misc {
    pub fn new(app_id: ApplicationId, bot_permissions: u64, container: cmp2::container::RefContainer) -> Self {
        Self {
            app_id,
            bot_permissions,
            container,
        }
    }
    #[event(Ready)]
    async fn on_ready(&self, ctx: &Context, ready: &ReadyEvent) {
        let perms = Permissions::from_bits(self.bot_permissions)
            .map(|v| {
                println!("Permission(s) demandé par le bot: {}", v);
                v
            })
            .unwrap_or_else(|| {
                println!("Permission du bot dans la configuration invalide. Utilisation de la permission par défaut.");
                Permissions::empty()
            });

        
        match ready.ready.user.invite_url(&ctx.http, perms).await {
            Ok(v) => println!("Invitation: {}", v),
            Err(e) => warn!("Lien d'invitation impossiblre à créer: {}", e.to_string()),
        }
    }
    #[command(description="Pong!")]
    async fn ping(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        if let Err(e) = app_cmd.direct_response(ctx, message::success("Pong!")).await {
            error!("ping: Erreur lors de la réponse: {}", e);
        }
    }
}