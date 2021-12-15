//! Générateurs de commandes slash via notre API de commandes.

use serenity::{builder::{CreateApplicationCommand, CreateApplicationCommandOption}, model::interactions::application_command::ApplicationCommandOptionType};
use super::command_parser::{self as cmd, Named};

pub fn register_root_with_perm(node: &cmd::Node, permission: bool) -> Vec<CreateApplicationCommand> {
    let group_iter = node.groups.list()
        .map(|group|{
            let mut app_cmd = CreateApplicationCommand::default();
            app_cmd.name(group.name());
            app_cmd.default_permission(permission);
            group.help().map(|help| app_cmd.description(help));
            app_cmd.set_options(group.groups().list().map(|g| register_group(g)).chain(group.commands().list().map(|cmd| register_command(cmd))).collect());
            app_cmd
        });
    let command_iter = node.commands.list()
        .map(|command| {
            let mut app_cmd = CreateApplicationCommand::default();
            app_cmd.name(command.name());
            app_cmd.default_permission(permission);
            command.help().map(|help| app_cmd.description(help));
            app_cmd.set_options(command.params().iter().map(|param| register_argument(param)).collect());
            app_cmd
        });
        group_iter.chain(command_iter).collect()
}
fn register_group(group: &cmd::Group) -> CreateApplicationCommandOption{
    let mut app_cmd = CreateApplicationCommandOption::default();
    app_cmd.name(group.name()).kind(ApplicationCommandOptionType::SubCommandGroup);

    group.help().map(|help| app_cmd.description(help));
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
    command.help().map(|help| app_cmd.description(help));
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
    argument.help().map(|help| app_cmd.description(help));
    app_cmd
}