use std::{slice::Iter, fmt::Display};

use serenity::{model::{interactions::application_command::ApplicationCommandOptionType}, builder::{CreateApplicationCommands, CreateApplicationCommandOption, CreateApplicationCommand}};

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
    pub fn iter_flat(&'static self) -> IterFlatNode {
        IterFlatNode::new(self)
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
impl Display for ChildNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (noeud) : {}", self.name, self.description)
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

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (commande) : {}", self.name, self.description)
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

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}) : {}", self.name, self.type_ as u8, self.description)
    }
}

pub struct IterFlatNode
{
    // node: &'static Node,
    // nodes_chain: Vec<IterFlatCommand>,
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