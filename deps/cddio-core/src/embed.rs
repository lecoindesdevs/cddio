use serenity::{
    model::{
        id::{GuildId, UserId, RoleId}, 
        application::{
            interaction::{
                InteractionResponseType,
                application_command::{ApplicationCommandInteraction, CommandDataOption, CommandData}
            },
            command::CommandOptionType
        }
    }, 
    client::Context, 
    builder::EditInteractionResponse
};
use crate::message::Message;

/// Helper to parse an application command.
#[derive(Clone)]
enum CommandType<'b> {
    Command(&'b CommandData),
    Option(&'b CommandDataOption)
}

impl<'a> CommandType<'a> {
    pub fn get_argument(&'a self, name: &str) -> Option<&'a CommandDataOption> {
        match self {
            CommandType::Command(command) => {
                command.options.iter().find(|option| option.name == name)
            },
            CommandType::Option(option) => {
                option.options.iter().find(|option| option.name == name)
            }
        }
    }
}

/// # Delayed interaction response
/// 
/// Once created by [`Self::new`] a delayed response is sent in response to the application command.
/// You must call [`Self::send`] to send the finished response.
/// 
/// See [`Self::new`] for more information.
pub struct DelayedResponse<'a> {
    pub message: Option<Message>,
    ctx: &'a Context,
    app_cmd: ApplicationCommandEmbed<'a>
}

impl<'a> DelayedResponse<'a> {
    /// Create a new delayed response
    /// 
    /// Send a delayed response to the application command
    pub async fn new(ctx: &'a Context, app_cmd: ApplicationCommandEmbed<'a>, ephemeral: bool) -> serenity::Result<DelayedResponse<'a>> {
        Self::send_new_response(ctx, app_cmd.0, ephemeral).await.or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e)
        })?;
        
        Ok(DelayedResponse {
            message: None,
            ctx,
            app_cmd
        })
    }
    /// Returns the embedded message. If the message is not yet created, it will be created.
    pub fn message(&mut self) -> &mut Message {
        if let None = self.message {
            self.message = Some(Message::with_text(String::new()));
        }
        match self.message {
            Some(ref mut message) => message,
            None => unreachable!("Message already created")
        }
    }
    /// Consume the response and send it to edit the interaction
    pub async fn send(mut self) -> serenity::Result<()> {
        let result = Self::edit_response(self.ctx, self.app_cmd.0, &self.message).await.or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e)
        });
        self.message = None;
        result
    }
    /// Consume the response and send a message to edit the interaction
    pub async fn send_message(mut self, msg: Message) -> serenity::Result<()> {
        self.message = Some(msg);
        self.send().await
    }
    /// Edit the interaction without taking account of the message
    pub async fn edit_and_send<F>(self, f: F) -> serenity::Result<()> where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse
        {
        self.app_cmd.0.edit_original_interaction_response(self.ctx, f).await.and(Ok(()))
    }
    async fn send_new_response(ctx: &Context, app_cmd: &ApplicationCommandInteraction, ephemeral: bool) -> serenity::Result<()> {
        app_cmd.create_interaction_response(ctx, |resp|{
            resp
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.ephemeral(ephemeral)
                })
        }).await
    }
    async fn edit_response(ctx: &Context, app_cmd: &ApplicationCommandInteraction, msg: &Option<Message>) -> serenity::Result<()> {
        app_cmd.edit_original_interaction_response(ctx, |resp|{
            match msg {
                Some(msg) => *resp = msg.into(),
                None => ()
            };
            resp
        }).await.and(Ok(()))
    }
}

impl<'a> Drop for DelayedResponse<'a> {
    fn drop(&mut self) {
        if let Some(msg) = &self.message {
            println!("Delayed message not sent: {:?}", msg);
        }
    }
}

/// # Conteneur d'application command
/// 
/// Lorsque le bot reçoit une commande via un événement de type interaction, cette structure facilite
/// l'accès aux données d'une struct ApplicationCommandInteraction. La commande et ses arguments
/// pouvant se situer à différent niveau en fonction des sous groupes, [`get_argument`] permet
/// d'obtenir directement les arguments.
/// 
/// [`get_argument`]: `Self::get_argument`
#[derive(Clone)]
pub struct ApplicationCommandEmbed<'a>(pub &'a ApplicationCommandInteraction, CommandType<'a>);

impl<'a> ApplicationCommandEmbed<'a> {
    /// Create a new application command embed
    /// 
    /// La (sous) commande est recherchée dans la commande principale, puis dans les options.
    pub fn new(interaction: &'a ApplicationCommandInteraction) -> Self {
        let mut command = CommandType::Command(&interaction.data);
        loop {
            let options = match command {
                CommandType::Command(data) => &data.options,
                CommandType::Option(data) => &data.options
            };
            if options.len() == 0 {
                break;
            }
            if let Some(cmd) = options.iter().find(|option| option.kind == CommandOptionType::SubCommand || option.kind == CommandOptionType::SubCommandGroup) {
                command = CommandType::Option(cmd);
            } else {
                break;
            }
        }
        ApplicationCommandEmbed(interaction, command)
    }
    pub fn fullname_vec<'b>(&'b self) -> Vec<&'b str> {
        let mut names = vec![self.0.data.name.as_str()];
        let mut cmd = self.0.data.options.first();
        // s'inspirer de la fonction get_command pour produire le nom
        while let Some(&CommandDataOption{ref name, ref options, kind: CommandOptionType::SubCommandGroup | CommandOptionType::SubCommand, ..}) = cmd {
            names.push(name.as_str());
            cmd = options.first();
        }
        names
    }
    /// Retourne le nom de la commande complète.
    /// 
    /// Ca inclut le nom des sous groupes et de la commande tel que `groupe.sous_groupe.commande`
    pub fn fullname(&self) -> String {
        self.fullname_vec().join(".")
    }
    /// Retourne l'id du serveur sur lequel la commande a été effectuée.
    pub fn get_guild_id(&self) -> Option<GuildId> {
        self.0.guild_id
    }
    /// Cherche et retourne l'argument `name`.
    pub fn get_argument(&'a self, name: &str) -> Option<&'a CommandDataOption> {
        self.1.get_argument(name)
    }

    pub async fn delayed_response<'b>(&'b self, ctx: &'b Context, ephemeral: bool) -> serenity::Result<DelayedResponse<'b>> {
        DelayedResponse::new(ctx, (*self).clone(), ephemeral).await
    }

    pub async fn direct_response(&self, ctx: &Context, msg: Message) -> serenity::Result<()> {
        self.0.create_interaction_response(ctx, |resp|{

            resp.kind(InteractionResponseType::ChannelMessageWithSource);
            *resp = msg.into();
            resp
        }).await
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mentionable {
    User(UserId),
    Role(RoleId),
}