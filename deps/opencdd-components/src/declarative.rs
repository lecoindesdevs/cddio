pub struct Node {
    pub children: &'static [ChildNode],
    pub commands: &'static [Command]
}
pub struct ChildNode {
    pub name: &'static str,
    pub description: &'static str,
    pub node: Node,
}
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [Argument],
}
pub struct Argument {
    pub name: &'static str,
    pub type_: serenity :: model :: interactions :: application_command :: ApplicationCommandOptionType,
    pub description: &'static str,
    pub optional: bool,
}

pub trait ComponentDeclarative {
    fn declarative(&self) -> &'static Node;
}