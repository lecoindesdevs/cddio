use serenity::model::id::ChannelId;

use super::{cmp, message};
/// Envoie un message d'erreur qui indique que l'envoyeur n'a pas la permission dans le channel.
pub async fn no_perm<Ch: Into<ChannelId>>(ctx: &cmp::Context, channel: Ch) -> serenity::Result<()> {
    error(ctx, channel, "Vous n'avez pas la permission d'utiliser cette commande.").await
}
/// Envoie un message d'erreur dans le channel.
pub async fn error<Ch: Into<ChannelId> ,S: ToString>(ctx: &cmp::Context, channel: Ch, error_message: S) -> serenity::Result<()> {
    match channel.into().send_message(ctx, |m| {
        *m = message::error(error_message);
        m
    }).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
/// Envoie un message d'erreur dans le channel.
pub async fn success<Ch: Into<ChannelId> ,S: ToString>(ctx: &cmp::Context, channel: Ch, success_message: S) -> serenity::Result<()> {
    match channel.into().send_message(ctx, |m| {
        *m = message::success(success_message);
        m
    }).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
