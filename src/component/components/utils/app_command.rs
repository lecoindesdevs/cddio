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

pub struct ApplicationCommandEmbed<'a>(pub &'a ApplicationCommandInteraction, CommandType<'a>);

impl<'a> ApplicationCommandEmbed<'a> {
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
            if let Some(cmd) = options.iter().find(|option| option.kind == ApplicationCommandOptionType::SubCommand) {
                command = CommandType::Option(cmd);
            } else {
                break;
            }
        }
        ApplicationCommandEmbed(interaction, command)
    }
    pub fn fullname(&self) -> String {
        let mut names = vec![self.0.data.name.as_str()];
        let mut cmd = self.0.data.options.first();
        // s'inspirer de la fonction get_command pour produire le nom
        while let Some(opt) = cmd {
            names.push(opt.name.as_str());
            if opt.kind == ApplicationCommandOptionType::SubCommand {
                return names.join(".");
            }
            cmd = opt.options.first();
        }
        names.join(".")
    }
    pub fn get_guild_id(&self) -> Option<GuildId> {
        self.0.guild_id
    }
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
        self.1.get_argument(name)
    }
}

macro_rules! unwrap_argument {
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(ref v) => {
                match v.resolved {
                    Some(ref v) => {
                        match v {
                            serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ (v) => v,
                            _ => return Err(format!("{}: Mauvais type d'argument. {} attendu", $name, stringify!($typ)))
                        }
                    },
                    None => return Err("Erreur de syntaxe".to_string())
                }
            },
            None => return Err(format!("{}: Paramètre requis manquant", $name))
        };
    };
}
macro_rules! unwrap_optional_argument {
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(ref v) => {
                match v.resolved {
                    Some(ref v) => {
                        match v {
                            serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ (v) => Some(v),
                            _ => return Err(format!("{}: Mauvais type d'argument. {} attendu", $name, stringify!($typ)))
                        }
                    },
                    None => return Err(format!("{}: Impossible à résoudre l'objet", $name))
                }
            },
            None => None
        };
    };
}
macro_rules! get_argument {
    ($app_command:expr, $name:expr, $typ:ident) => {
        match $app_command.get_argument($name) {
            Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::$typ(s)),
                ..
            }) => Some(s),
            _ => None
        };
    };
}
pub(crate) use {unwrap_argument, unwrap_optional_argument, get_argument};