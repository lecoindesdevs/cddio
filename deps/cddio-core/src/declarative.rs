use std::{slice::Iter, fmt::Display};

use serenity::{model::{interactions::application_command::ApplicationCommandOptionType}, builder::{CreateApplicationCommands, CreateApplicationCommandOption, CreateApplicationCommand}};
use crate::message::{self, ToMessage};

/// The component declaration trait.
/// 
/// This trait is used to declare groups and root slash commands from a component.
/// 
/// This trait is required if at least one command is declared. 
/// It is not required if the component only handles events.
pub trait ComponentDeclarative{
    fn declarative(&self) -> Option<&'static Node> {
        None
    }
}
/// Node of the component declaration.
/// 
/// The component declaration is a tree of nodes. 
/// Each node contains a reference to other nodes ([`ChildNode`]) and to commands [`Command`].
/// 
/// `children` and `commands` shoudl be declared const/static in the program and per component.
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
    pub fn iter_flat(&'static self) -> IterFlatNode {
        IterFlatNode::new(self)
    }
    pub fn to_markdown(&'static self) -> String {
        let mut s = String::new();
        self.iter_flat()
            .filter_map(|(name, v)| match v {
                IterType::Command(command) => Some((name, command)),
                _ => None
            })
            .for_each(|(fullname, item)| {
                s.push_str(&format!("## /{}\n\n{}\n", fullname, item.to_markdown()));
            });
        s
    }
}
/// Node description data
pub struct ChildNode {
    /// The name of the node.
    /// 
    /// This name is used name the group application command the node is related to.
    pub name: &'static str,
    /// The node description.
    pub description: &'static str,
    /// The node children.
    pub node: Node,
}
impl ChildNode {
    /// Iterate over the node to extract only commands.
    pub fn iter_flat(&'static self) -> IterFlatNode {
        IterFlatNode::new(&self.node)
    }
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
impl Display for ChildNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (noeud) : {}", self.name, self.description)
    }
}

impl ToMessage for &'static ChildNode {
    fn to_message(&self) -> message::Message {
        let cmds = self.iter_flat()
            .filter_map(|(fullname, iter_type)| {
                match iter_type {
                    IterType::Command(cmd) => Some(format!("**{}**: {}", fullname, cmd.description)),
                    _ => None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut embed = message::Embed::default();
        embed
            .title(format!("Groupe {}", self.name))
            .description(self.description)
            .color(message::COLOR_SUCCESS)
            .field("Commandes", cmds, false);
        
        message::Message { message: String::new(), embeds: vec![embed], ephemeral: false }
    }
}
/// Command description data
pub struct Command {
    /// The name of the command.
    pub name: &'static str,
    /// The command description.
    pub description: &'static str,
    /// The command arguments. Can be empty.
    pub args: &'static [Argument],
}

impl Command {
    pub fn to_markdown(&'static self) -> String {
        let mut s = format!("{}\n\n", self.description);
        if !self.args.is_empty() {
            s.push_str("### Arguments\n\n");
            for arg in self.args {
                s.push_str(&format!("* {}\n", arg.to_markdown()));
            }
        }
        s
    }
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

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (commande) : {}", self.name, self.description)
    }
}
impl message::ToMessage for Command {
    fn to_message(&self) -> message::Message {
        let mut embed = message::Embed::default();
        embed
            .title(format!("Commande {}", self.name))
            .description(self.description)
            .color(message::COLOR_SUCCESS);
        if !self.args.is_empty() {
            let title = if self.args.len() == 1 {"Argument"} else {"Arguments"};
            let args_str = self.args.iter()
                .map(|arg| {
                    let opt_str = if arg.optional { " (optionnel)" } else { "" };
                    format!("**{}**{}: {}", arg.name, opt_str, arg.description)
                })
                .collect::<Vec<_>>()
                .join("\n");
            embed.field(title, args_str, false);
        }
        message::Message { message: String::new(), embeds: vec![embed], ephemeral: false }
    }
}
/// Argument description data
pub struct Argument {
    /// The name of the argument.
    pub name: &'static str,
    /// The argument type. Restricted to [`ApplicationCommandOptionType`].
    pub type_: serenity::model::interactions::application_command::ApplicationCommandOptionType,
    /// The argument description.
    pub description: &'static str,
    /// Whether the argument is optional to the command.
    pub optional: bool,
}
impl Argument {
    pub fn to_markdown(&'static self) -> String {
        let opt_str = if self.optional { " (optionnel)" } else { "" };
        format!("**{}**{}: {}", self.name, opt_str, self.description)
    }
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

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

/// Flat node iterator.
/// 
/// Iterate over all the node tree recursively.
/// It begins to iterate through children (if any) then the commands (if any)
/// 
/// [`next()`] returns a tuple of complete path to the current item 
/// and the current item ([Node] or [Command])
/// 
/// [`next()`]: IterFlatNode::next
pub struct IterFlatNode
{
    name: Option<&'static str>,
    children: Iter<'static, ChildNode>,
    current_child: Option<Box<IterFlatNode>>,
    commands: Iter<'static, Command>,
}

impl IterFlatNode
{
    pub fn new(node: &'static Node) -> Self {
        IterFlatNode {
            name: None,
            children: node.children.iter(),
            current_child: None,
            commands: node.commands.iter(),
        }
    }
    fn from_child(node: &'static Node, name: &'static str) -> Self {
        IterFlatNode {
            name: Some(name),
            children: node.children.iter(),
            current_child: None,
            commands: node.commands.iter(),
        }
    }
}

/// Item returned by [`IterFlatNode::next()`]. 
/// Can be either a [Node] or a [Command].
pub enum IterType {
    Node(&'static ChildNode),
    Command(&'static Command)
}

impl Display for IterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterType::Node(node) => write!(f, "{}", node),
            IterType::Command(command) => write!(f, "{}", command),
        }
    }
}
impl IterType {
    pub fn name(&self) -> &'static str {
        match self {
            IterType::Node(node) => node.name,
            IterType::Command(command) => command.name,
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            IterType::Node(node) => node.description,
            IterType::Command(command) => command.description,
        }
    }
}

impl Iterator for IterFlatNode
{
    /// Tuple to the complete path of the current item and the current item
    type Item = (String, IterType);
    fn next(&mut self) -> Option<Self::Item> {
        let fullname = |other: &str| {
            match self.name {
                Some(name) => format!("{} {}", name, other),
                None => other.into(),
            }
        };
        loop {
            if let Some(node) = &mut self.current_child {
                match node.next() {
                    Some((name, v)) => return Some((fullname(&name), v)),
                    None => {
                        self.current_child = None;
                        continue;
                    }
                }
            } else if let Some(child_node) = self.children.next() {
                self.current_child = Some(Box::new(IterFlatNode::from_child(&child_node.node, child_node.name)));
                return Some((fullname(child_node.name.into()), IterType::Node(child_node)));
            } else if let Some(command) = self.commands.next() {
                return Some((fullname(command.name.into()), IterType::Command(command)));
            } else {
                return None;
            }
        }
    }
}