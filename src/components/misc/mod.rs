//! Miscellaneous commands and events.

use crate::{log_error, log_warn};
use cddio_core::{self as core, ApplicationCommandEmbed, message};
use cddio_macros::component;
use serenity::{
    client::Context, 
    model::{id::ApplicationId, permissions::Permissions, event::ReadyEvent}
};

pub struct Misc {
    app_id: ApplicationId,
    bot_permissions: u64,
    container: core::container::RefContainer,
}

#[component]
impl Misc {
    pub fn new(app_id: ApplicationId, bot_permissions: u64, container: core::container::RefContainer) -> Self {
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
                println!("Permission du bot dans la configuration invalide. Utilisation des permissions par défaut.");
                Permissions::default()
            });

        
        match ready.ready.user.invite_url(&ctx.http, perms).await {
            Ok(v) => println!("Invitation: {}", v),
            Err(e) => log_warn!("Lien d'invitation impossiblre à créer: {}", e.to_string()),
        }
    }
    #[command(description="Pong!")]
    async fn ping(&self, ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>) {
        if let Err(e) = app_cmd.direct_response(ctx, message::success("Pong!")).await {
            log_error!("ping: Erreur lors de la réponse: {}", e);
        }
    }
}