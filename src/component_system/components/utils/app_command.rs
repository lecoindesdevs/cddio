use serenity::model::{id::GuildId, interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType, ApplicationCommandInteractionData}};

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

/// # Conteneur d'application command
/// 
/// Lorsque le bot recoie une commande via un evenement d'interaction, cette structure est un utilitaire
/// pour accéder aux données d'une struct ApplicationCommandInteraction plus facilement. La commande et 
/// ses arguments pouvant se situer à différent niveau en fonction des sous groupes, [`get_argument`] 
/// permet d'obtenir directement les arguments.
/// 
/// [`get_argument`]: `Self::get_argument`
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
    /// Retourne le nom de la commande complète.
    /// 
    /// Ca inclut le nom des sous groupes et de la commande tel que `groupe.sous_groupe.commande`
    pub fn fullname(&self) -> String {
        let mut names = vec![self.0.data.name.as_str()];
        let mut cmd = self.0.data.options.first();
        // s'inspirer de la fonction get_command pour produire le nom
        while let Some(&ApplicationCommandInteractionDataOption{ref name, ref options, kind: ApplicationCommandOptionType::SubCommandGroup | ApplicationCommandOptionType::SubCommand, ..}) = cmd {
            names.push(name.as_str());
            cmd = options.first();
        }
        names.join(".")
    }
    /// Retourne l'id du serveur sur lequel la commande a été effectuée.
    pub fn get_guild_id(&self) -> Option<GuildId> {
        self.0.guild_id
    }
    /// Cherche et retourne l'argument `name`.
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
        self.1.get_argument(name)
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