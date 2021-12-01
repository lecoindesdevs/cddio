use serenity::builder::{CreateEmbed, CreateInteractionResponse, CreateInteractionResponseData, CreateMessage};
use serenity::utils::Colour;

/// Interface de création de message
/// 
/// Utile pour passer les mêmes informations de d'envoi d'un message 
/// aux différentes API (commandes par message ou slash)
/// 
/// Interface succeptible de changer en fonction des besoins
pub struct Message{
    pub message: String,
    pub embed: Option<CreateEmbed>,
    pub ephemeral: bool,
}

impl Message {
    pub fn new(message: String) -> Self {
        Message {
            message,
            ..Default::default()
        }
    }
}
impl Default for Message {
    fn default() -> Self {
        Message {
            message: String::new(),
            embed: None,
            ephemeral: false,
        }
    }
}
impl From<Message> for CreateMessage<'static> {
    fn from(message: Message) -> Self {
        let mut res = CreateMessage::default();
        res.content(message.message);
        if let Some(embed) = message.embed {
            res.embed(|e| {*e = embed; e});
        }
        res
    }
}
impl From<Message> for CreateInteractionResponse {
    fn from(message: Message) -> Self {
        use serenity::model::interactions::{InteractionResponseType, InteractionApplicationCommandCallbackDataFlags};
        let mut response = CreateInteractionResponse::default();
        response.interaction_response_data(|data|{
            if message.ephemeral {
                data.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            }
            data.content(message.message);
            if let Some(embed) = message.embed {
                data.create_embed(|e| {*e = embed; e});
            }
            data
        });
        response.kind(InteractionResponseType::ChannelMessageWithSource);
        response
    }
}
/// Génère un message d'erreur
pub fn error<S: ToString>(error_message: S) -> Message {
    custom_embed("Attention", error_message, 0xFF0000)
}
/// Génère un message de succès
pub fn success<S: ToString>(success_message: S) -> Message {
    custom_embed("Effectué", success_message, 0x1ed760)
}
/// Génère un message personnalisé
pub fn custom_embed<S1, S2, C>(title:S1, message: S2, color: C) -> Message
    where 
    S1: ToString, 
    S2: ToString,
    C: Into<Colour>
{
    let mut embed = CreateEmbed::default();
    embed
        .title(title)
        .description(message)
        .color(color);
    Message {
        embed: Some(embed),
        ..Default::default()
    }
}
// pub fn success