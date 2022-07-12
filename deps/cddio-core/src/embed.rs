use serenity::{model::{id::{GuildId, UserId, RoleId}, interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType, ApplicationCommandInteractionData}}, client::Context, builder::{CreateInteractionResponse, EditInteractionResponse}};

use crate::message::Message;
#[derive(Clone)]
enum CommandType<'b> {
    Command(&'b ApplicationCommandInteractionData),
    Option(&'b ApplicationCommandInteractionDataOption)
}

impl<'a> CommandType<'a> {
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
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

pub struct DelayedResponse<'a> {
    pub message: Option<Message>,
    ctx: &'a Context,
    app_cmd: ApplicationCommandEmbed<'a>
}

impl<'a> DelayedResponse<'a> {
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
    pub fn message(&mut self) -> &mut Message {
        if let None = self.message {
            self.message = Some(Message::with_text(String::new()));
        }
        match self.message {
            Some(ref mut message) => message,
            None => unreachable!("Message already created")
        }
    }
    pub async fn send(mut self) -> serenity::Result<()> {
        let result = Self::edit_response(self.ctx, self.app_cmd.0, &self.message).await.or_else(|e| {
            eprintln!("Cannot create response: {}", e);
            Err(e)
        });
        self.message = None;
        result
    }
    pub async fn send_message(mut self, msg: Message) -> serenity::Result<()> {
        self.message = Some(msg);
        self.send().await
    }
    pub async fn edit_and_send<F>(self, f: F) -> serenity::Result<()> where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse
        {
        self.app_cmd.0.edit_original_interaction_response(self.ctx, f).await.and(Ok(()))
    }
    async fn send_new_response(ctx: &Context, app_cmd: &ApplicationCommandInteraction, ephemeral: bool) -> serenity::Result<()> {
        use serenity::model::interactions::InteractionResponseType;
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
    /// Créer un conteneur d'application command
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
            if let Some(cmd) = options.iter().find(|option| option.kind == ApplicationCommandOptionType::SubCommand || option.kind == ApplicationCommandOptionType::SubCommandGroup) {
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
        while let Some(&ApplicationCommandInteractionDataOption{ref name, ref options, kind: ApplicationCommandOptionType::SubCommandGroup | ApplicationCommandOptionType::SubCommand, ..}) = cmd {
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
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
        self.1.get_argument(name)
    }

    pub async fn delayed_response<'b>(&'b self, ctx: &'b Context, ephemeral: bool) -> serenity::Result<DelayedResponse<'b>> {
        DelayedResponse::new(ctx, (*self).clone(), ephemeral).await
    }

    pub async fn direct_response(&self, ctx: &Context, msg: Message) -> serenity::Result<()> {
        self.0.create_interaction_response(ctx, |resp|{
            use serenity::model::interactions::InteractionResponseType;
            resp.kind(InteractionResponseType::ChannelMessageWithSource);
            *resp = msg.into();
            resp
        }).await
    }

}
/// Helper pour accéder à un argument d'une commande
/// 
/// Fait plusieurs vérifications pour obtenir l'argument et simplifie la lecture du code
macro_rules! get_argument_result {
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(ref v) => {
                match v.resolved {
                    Some(ref v) => {
                        match v {
                            serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ (v) => Ok(v),
                            _ => Err(format!("{}: Mauvais type d'argument. {} attendu", $name, stringify!($typ)))
                        }
                    },
                    None => Err("Erreur de syntaxe".to_string())
                }
            },
            None => Err(format!("{}: Paramètre requis manquant", $name))
        }
    };
}
/// Helper pour accéder à un argument d'une commande s'il existe
/// 
/// Fait plusieurs vérifications pour obtenir l'argument et simplifie la lecture du code
macro_rules! get_optional_argument_result {
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(ref v) => {
                match v.resolved {
                    Some(ref v) => {
                        match v {
                            serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ (v) => Ok(Some(v)),
                            _ => Err(format!("{}: Mauvais type d'argument. {} attendu", $name, stringify!($typ)))
                        }
                    },
                    None => Err("Erreur de syntaxe".to_string())
                }
            },
            None => Ok(None)
        }
    };
}
/// Helper pour accéder à un argument d'une commande
/// 
/// Contraitement à [`get_argument_result!`] et [`get_optional_argument_result!`], cette macro ne fait 
/// pas de vérification superflu et retourne l'argument de la commande si les critères sont remplis.
macro_rules! get_argument {
    ($app_command:expr, $name:expr, User) => {
        match $app_command.get_argument($name) {
            Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::User(a, b)),
                ..
            }) => Some((a, b)),
            _ => None
        }
    };
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ(s)),
                ..
            }) => Some(s),
            _ => None
        }
    };
}
#[allow(unused_imports)]
pub(crate) use {get_argument_result, get_optional_argument_result, get_argument};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mentionable {
    User(UserId),
    Role(RoleId),
}