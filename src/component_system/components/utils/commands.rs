pub use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::{interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption, ApplicationCommandInteractionData}, id::{UserId, ChannelId, RoleId}};

use crate::component_system::command_parser::matching;
#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(UserId),
    Channel(ChannelId),
    Role(RoleId),
    Number(f64),
    Mention(u64),
}

impl From<ApplicationCommandInteractionDataOptionValue> for Value {
    fn from(value: ApplicationCommandInteractionDataOptionValue) -> Self {
        match value {
            ApplicationCommandInteractionDataOptionValue::String(s) => Value::String(s),
            ApplicationCommandInteractionDataOptionValue::Integer(i) => Value::Integer(i),
            ApplicationCommandInteractionDataOptionValue::Boolean(b) => Value::Boolean(b),
            ApplicationCommandInteractionDataOptionValue::User(u, _) => Value::User(u.id),
            ApplicationCommandInteractionDataOptionValue::Channel(c) => Value::Channel(c.id),
            ApplicationCommandInteractionDataOptionValue::Role(r) => Value::Role(r.id),
            ApplicationCommandInteractionDataOptionValue::Number(n) => Value::Number(n),
            _ => panic!("Unsupported value type"),
        }
    }
}
#[derive(Debug)]
pub struct Command {
    pub path: Vec<String>,
    pub args: Vec<Argument>,
}
#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub value: Value,
}
impl Command {
    pub fn new(path: Vec<String>, args: Vec<Argument>) -> Self {
        Self { path, args }
    }
    pub fn fullname(&self) -> String {
        self.path.join(".")
    }
    pub fn get_argument(&self, name: &str) -> Option<&Argument> {
        self.args.iter().find(|arg| arg.name == name)
    } 
}

pub trait ToCommand {
    fn to_command(&self) -> Command;
}

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

impl ToCommand for ApplicationCommandInteraction {
    fn to_command(&self) -> Command {
        use serenity::model::interactions::application_command::{ApplicationCommandOptionType};
        
        let mut command = CommandType::Command(&self.data);
        let mut path = vec![self.data.name.clone()];
        loop {
            let options = match command {
                CommandType::Command(data) => &data.options,
                CommandType::Option(data) => &data.options
            };
            if options.len() == 0 {
                break;
            }
            if let Some(cmd) = options.iter().find(|option| option.kind == ApplicationCommandOptionType::SubCommand || option.kind == ApplicationCommandOptionType::SubCommandGroup) {
                path.push(cmd.name.clone());
                command = CommandType::Option(cmd);
            } else {
                break;
            }
        }
        let options = match command {
            CommandType::Command(&ApplicationCommandInteractionData { ref options, .. }) => options,
            CommandType::Option(&ApplicationCommandInteractionDataOption { ref options, .. }) => options
        };
        let args = options.iter().filter_map(|option| {
            let value = match &option.resolved {
                Some(value) => value.clone(),
                None => return None
            };
            Some(Argument {
                name: option.name.clone(),
                value: value.into()
            })
        }).collect();
        Command::new(path, args)
    }
}

impl<'a> ToCommand for matching::Command<'a> {
    fn to_command(&self) -> Command {
        
        let path = self.path.iter().map(|part| part.to_string()).collect();
        let args = self.params.iter().filter_map(|arg| {
            use serenity::model::interactions::application_command::ApplicationCommandOptionType::*;
            Some(Argument {
                name: arg.name.to_string(),
                value: match arg.kind {
                    String => Value::String(arg.value.to_string()),
                    Integer => match arg.value.parse::<i64>() {
                        Ok(value) => Value::Integer(value),
                        Err(_) => return None
                    },
                    Boolean => match arg.value {
                        "vrai"|"true"|"1" => Value::Boolean(true),
                        _ => Value::Boolean(false),
                    },
                    User => match arg.value.parse::<u64>() {
                        Ok(value) => Value::User(UserId(value)),
                        Err(_) => return None
                    },
                    Channel => match arg.value.parse::<u64>() {
                        Ok(value) => Value::Channel(ChannelId(value)),
                        Err(_) => return None
                    },
                    Role => match arg.value.parse::<u64>() {
                        Ok(value) => Value::Role(RoleId(value)),
                        Err(_) => return None
                    },
                    Mentionable => match arg.value.parse::<u64>() {
                        Ok(value) => Value::Mention(value),
                        Err(_) => return None
                    },
                    Number => match arg.value.parse::<f64>() {
                        Ok(value) => Value::Number(value),
                        Err(_) => return None
                    },
                    _ => return None
                }
            })
        }).collect();
        
        Command::new(path, args)
    }
}