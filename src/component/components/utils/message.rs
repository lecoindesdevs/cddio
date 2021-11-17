use serenity::builder::CreateMessage;
use serenity::utils::Colour;

pub fn error<S: ToString>(error_message: S) -> CreateMessage<'static> {
    custom_embed("Attention", error_message, 0xFF0000)
}
pub fn success<S: ToString>(success_message: S) -> CreateMessage<'static> {
    custom_embed("Effectué", success_message, 0x1ed760)
}
pub fn custom_embed<S1, S2, C>(title:S1, message: S2, color: C) -> CreateMessage<'static> 
    where 
    S1: ToString, 
    S2: ToString,
    C: Into<Colour>
{
    let mut msg = CreateMessage::default();
    msg.embed(|embed| embed
        .title(title)
        .description(message)
        .color(color)
    );
    msg
}
// pub fn success