# CDDIO MACROS

## Simple example

```rust
use cddio_core::{ApplicationCommandEmbed, message};
use serenity::{
    client::Context,
    event::*,
    model::{
        event::ReadyEvent,
        id::ChannelId
    }
}

struct MyComponent;

#[component]
impl MyComponent {
    /// Nom de la commande Discord: ping
    /// Arguments: (aucun)
    /// Description: Renvoie un message 'Pong!'
    #[command(name="ping", description="Renvoie un message 'Pong!'")]
    async fn ping_cmd(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>)
    {
        if let Err(e) = app_cmd.direct_response(ctx, message::success("Pong!")).await {
            log_error!("ping: Erreur lors de la réponse: {}", e);
        }
    }
    /// Nom de la commande Discord: creer_embed
    /// Arguments: 
    ///     - titre (obligatoire): type Texte,      Titre de l'embed
    ///     - contenu (obligatoire): type Texte,    Contenu de l'embed
    ///     - salon (optionnel): type ChannelId,    Salon où l'envoyer. Salon actuel par défaut
    ///     
    /// Description: Renvoie un message 'Pong!'
    #[command(description="Renvoie un message 'Pong!'")]
    async fn creer_embed(ctx: &Context, app_cmd: ApplicationCommandEmbed<'_>
        #[argument(description="Titre de l'embed")]
        titre: String,
        #[argument(name="contenu", description="Contenu de l'embed")]
        content: String,
        #[argument(description="Salon où l'envoyer. Salon actuel par défaut")]
        salon: Option<ChannelId>,
    )
    {
        /// implémentation...
    }
    /// Evenement appelé lorsque le bot est prêt
    #[event(Ready)]
    async fn on_ready(ctx: &Context, _evt_ready: &ReadyEvent)
    {
        println!("Bot is ready");
    }
}
```