use super::{cmd, cmp};
/// Envoie un message d'erreur qui indique que l'envoyeur n'a pas la permission dans le channel.
pub async fn no_perm(_ctx: &cmp::Context, msg: &cmp::Message) -> serenity::Result<()> {
    match msg.channel_id.send_message(&_ctx.http, |m|
        m.embed(|embed| {
            embed
                .title("Attention")
                .description(format!("Vous n'avez pas la permission d'utiliser cette commande"))
                .color(0xFF0000)
        })
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
/// Envoie un message d'erreur dans le channel.
pub async fn error_message<S: AsRef<str>>(_ctx: &cmp::Context, msg: &cmp::Message, error_message: S) -> serenity::Result<()> {
    match msg.channel_id.send_message(&_ctx.http, |m|
        m.embed(|embed| {
            embed
                .title("Attention")
                .description( error_message.as_ref() )
                .color(0xFF0000)
        })
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
/// Envoie un message d'erreur dans le channel.
pub async fn success_message<S: AsRef<str>>(_ctx: &cmp::Context, msg: &cmp::Message, success_message: S) -> serenity::Result<()> {
    match msg.channel_id.send_message(&_ctx.http, |m|
        m.embed(|embed| {
            embed
                .title("Effectué")
                .description( success_message.as_ref() )
                .color(0x1ed760)
        })
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
