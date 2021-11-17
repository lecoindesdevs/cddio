use serenity::model::{id::GuildId, interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType}};


pub struct ApplicationCommand<'a>(&'a ApplicationCommandInteraction);

impl<'a> ApplicationCommand<'a> {
    pub fn new(interaction: &'a ApplicationCommandInteraction) -> Self {
        ApplicationCommand(interaction)
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
    pub fn get_command(&'a self) -> &'a ApplicationCommandInteractionDataOption {
        let mut cmd = self.0.data.options.first();
        while let Some(&ApplicationCommandInteractionDataOption{ref kind, ref options, ..}) = cmd {
            if kind == &ApplicationCommandOptionType::SubCommand {
                return cmd.unwrap();
            }
            if options.is_empty() {
                panic!("No subcommand found.\n{:?}", self.0);
            }
            cmd = options.first();
        }
        unreachable!()
    }
    pub fn get_argument(&'a self, name: &str) -> Option<&'a ApplicationCommandInteractionDataOption> {
        let command = self.get_command();
        command.options.iter().find(|opt| opt.name.as_str() == name)
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
pub(crate) use {unwrap_argument, unwrap_optional_argument};