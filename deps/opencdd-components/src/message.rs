use serenity::builder::{CreateEmbed, CreateInteractionResponse, EditInteractionResponse, CreateMessage};
use serenity::utils::Colour;
pub use serenity::builder::CreateEmbed as Embed;

pub trait ToMessage {
    fn to_message(&self) -> Message;
}

pub const COLOR_INFO: Colour = Colour(0x00C9FF);
pub const COLOR_SUCCESS: Colour = Colour(0x1ed760);
pub const COLOR_ERROR: Colour = Colour(0xFF0000);
pub const COLOR_WARN: Colour = Colour(0xFFB800);

/// Interface de création de message
/// 
/// Utile pour passer les mêmes informations de d'envoi d'un message 
/// aux différentes API (commandes par message ou slash)
/// 
/// Interface succeptible de changer en fonction des besoins
#[derive(Debug, Clone)]
pub struct Message{
    pub message: String,
    pub embeds: Vec<CreateEmbed>,
    pub ephemeral: bool,
}

impl Message {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_text(message: String) -> Self {
        Message {
            message,
            ..Default::default()
        }
    }
    pub fn set_ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = ephemeral;
        self
    }
    pub fn last_embed(&self) -> Option<&CreateEmbed> {
        self.embeds.last()
    }
    pub fn last_embed_mut(&mut self) -> Option<&mut CreateEmbed> {
        self.embeds.last_mut()
    }
}
impl Default for Message {
    fn default() -> Self {
        Self {
            message: String::new(),
            embeds: Vec::new(),
            ephemeral: false,
        }
    }
}
impl From<Message> for CreateMessage<'static> {
    fn from(message: Message) -> Self {
        let mut res = CreateMessage::default();
        res.content(message.message);
        res.add_embeds(message.embeds);
        res
    }
}
impl From<Message> for CreateInteractionResponse<'_> {
    fn from(message: Message) -> Self {
        use serenity::model::interactions::{InteractionResponseType, InteractionApplicationCommandCallbackDataFlags};
        let mut response = CreateInteractionResponse::default();
        response.interaction_response_data(|data|{
            if message.ephemeral {
                data.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            }
            data.content(message.message);
            data.set_embeds(message.embeds.into_iter());
            data
        });
        response.kind(InteractionResponseType::ChannelMessageWithSource);
        response
    }
}
impl From<&Message> for EditInteractionResponse {
    fn from(message: &Message) -> Self {
        let mut response = Self::default();
        response.content(&message.message);
        response.set_embeds(message.embeds.clone());
        response
    }
}
impl From<Message> for EditInteractionResponse {
    fn from(message: Message) -> Self {
        let mut response = Self::default();
        response.content(message.message);
        response.set_embeds(message.embeds);
        response
    }
}
/// Génère un message d'erreur
pub fn error<S: ToString>(error_message: S) -> Message {
    custom_embed("Erreur", error_message, COLOR_ERROR)
}
/// Génère un message d'avertissement
pub fn warn<S: ToString>(warn_message: S) -> Message {
    custom_embed("Attention", warn_message, COLOR_WARN)
}
/// Génère un message de succès
pub fn success<S: ToString>(success_message: S) -> Message {
    custom_embed("Effectué", success_message, COLOR_SUCCESS)
}
/// Génère un message d'information
pub fn info<S: ToString>(info_message: S) -> Message {
    custom_embed("Information", info_message, COLOR_INFO)
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
        embeds: vec![embed],
        ..Default::default()
    }
}
// pub fn success