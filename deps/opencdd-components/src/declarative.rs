use std::sync::Arc;

use serenity::{model::{id::GuildId, interactions::application_command::ApplicationCommandOptionType}, builder::{CreateApplicationCommands, CreateApplicationCommandOption, CreateApplicationCommand}, client::Context};

pub trait ComponentDeclarative{
    fn declarative(&self) -> Option<&'static Node> {
        None
    }
}

pub struct Node {
    pub children: &'static [ChildNode],
    pub commands: &'static [Command]
}
impl Node {
    pub fn add_application_command(&self, commands: &mut CreateApplicationCommands) {
        for child in self.children {
            commands.add_application_command(child.into());
        }
        for command in self.commands {
            commands.add_application_command(command.into());
        }
    }
}
pub struct ChildNode {
    pub name: &'static str,
    pub description: &'static str,
    pub node: Node,
}
impl From<&ChildNode> for CreateApplicationCommandOption {
    fn from(group: &ChildNode) -> Self {
        let mut app_cmd = CreateApplicationCommandOption::default();
        app_cmd
            .kind(ApplicationCommandOptionType::SubCommandGroup)
            .name(group.name)
            .description(group.description);
        for grp in group.node.children {
            app_cmd.add_sub_option(grp.into());
        }
        for cmd in group.node.commands {
            app_cmd.add_sub_option(cmd.into());
        }
        app_cmd
    }
}
impl From<&ChildNode> for CreateApplicationCommand {
    fn from(group: &ChildNode) -> Self {
        let mut app_cmd = CreateApplicationCommand::default();
        app_cmd.name(group.name);
        app_cmd.description(group.description);
        for grp in group.node.children {
            app_cmd.add_option(grp.into());
        }
        for cmd in group.node.commands {
            app_cmd.add_option(cmd.into());
        }
        app_cmd
    }
}
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [Argument],
}
impl From<&Command> for CreateApplicationCommandOption {
    fn from(command: &Command) -> Self {
        let mut app_cmd = CreateApplicationCommandOption::default();
        app_cmd
            .kind(ApplicationCommandOptionType::SubCommand)
            .name(command.name)
            .description(command.description);
        for arg in command.args {
            app_cmd.add_sub_option(arg.into());
        }
        app_cmd
    }
} 
impl From<&Command> for CreateApplicationCommand {
    fn from(command: &Command) -> Self {
        let mut app_cmd = CreateApplicationCommand::default();
        app_cmd
            .name(command.name)
            .description(command.description);
        for arg in command.args {
            app_cmd.add_option(arg.into());
        }
        app_cmd
    }
} 
pub struct Argument {
    pub name: &'static str,
    pub type_: serenity :: model :: interactions :: application_command :: ApplicationCommandOptionType,
    pub description: &'static str,
    pub optional: bool,
}
impl From<&Argument> for CreateApplicationCommandOption {
    fn from(argument: &Argument) -> Self {
        let mut app_cmd = CreateApplicationCommandOption::default();
        app_cmd
            .kind(argument.type_)
            .name(argument.name)
            .required(!argument.optional)
            .description(argument.description);
        app_cmd
    }
}

