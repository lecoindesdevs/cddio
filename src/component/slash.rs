use serenity::{builder::{CreateApplicationCommand, CreateApplicationCommandOption}, model::interactions::application_command::ApplicationCommandOptionType};

use super::command_parser::{self as cmd, Named};

pub fn register_root(group: &cmd::Group) -> CreateApplicationCommand {
    let mut app_cmd = CreateApplicationCommand::default();
    app_cmd.name(group.name());
    if let Some(help) = group.help() {
        app_cmd.description(help);
    }
        
    for grp in group.groups().list() {
        app_cmd.add_option(register_group(grp));
    }
    for cmd in group.commands().list() {
        app_cmd.add_option(register_command(cmd));
    }
    app_cmd
}
fn register_group(group: &cmd::Group) -> CreateApplicationCommandOption{
    let mut app_cmd = CreateApplicationCommandOption::default();
    app_cmd.name(group.name()).kind(ApplicationCommandOptionType::SubCommandGroup);

    if let Some(help) = group.help() {
        app_cmd.description(help);
    }
        
    for grp in group.groups().list() {
        app_cmd.add_sub_option(register_group(grp));
    }
    for cmd in group.commands().list() {
        app_cmd.add_sub_option(register_command(cmd));
    }
    app_cmd
}
fn register_command(command: &cmd::Command) -> CreateApplicationCommandOption {
    let mut app_cmd = CreateApplicationCommandOption::default();
    app_cmd.name(command.name()).kind(ApplicationCommandOptionType::SubCommand);
    if let Some(help) = command.help() {
        app_cmd.description(help);
    }
    for arg in command.params() {
        app_cmd.add_sub_option(register_argument(arg));
    }

    app_cmd
}
fn register_argument(argument: &cmd::Argument) -> CreateApplicationCommandOption {
    let mut app_cmd = CreateApplicationCommandOption::default();
    app_cmd.name(argument.name());
    app_cmd.required(argument.required());
    app_cmd.kind(argument.value_type());
    if let Some(help) = argument.help() {
        app_cmd.description(help);
    }
    app_cmd
}