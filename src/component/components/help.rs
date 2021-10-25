use std::{borrow::Cow, ops::Deref};

use serenity::async_trait;

use crate::component::{self as cmp, command_parser::{self as cmd, Named}};

pub struct Help {
    components: Vec<cmp::ArcComponent>
}
#[async_trait]
impl cmp::Component for Help {
    fn name(&self) -> &'static str {
        "help"
    }

    async fn command(&mut self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        self.r_command(fw_config, ctx, msg).await
    }

    async fn event(&mut self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        Ok(())
    }
}
type HelpString<'a> = Cow<'a, str>;
type HelpOptionString<'a> = Option<Cow<'a, str>>;
#[derive(Debug, Default)]
struct HelpInfo<'a> {
    name: HelpString<'a>,
    desc: Option<HelpString<'a>>,
    groups: Option<Vec<(HelpString<'a>, HelpOptionString<'a>)>>,
    commands: Option<Vec<(HelpString<'a>, HelpOptionString<'a>)>>,
    params: Option<Vec<(HelpString<'a>, HelpOptionString<'a>)>>,
    components: Option<Vec<String>>
}
trait ToCow<'a> {
    type Output: 'a;
    fn to_cow(self) -> Self::Output;
}
impl<'a> ToCow<'a> for String {
    type Output = Cow<'a, str>;
    fn to_cow(self) -> Self::Output {
        Cow::Owned(self)
    }
}
impl<'a> ToCow<'a> for &'a str {
    type Output = Cow<'a, str>;
    fn to_cow(self) -> Self::Output {
        Cow::Borrowed(self)
    }
}
impl<'a> ToCow<'a> for Option<&'a str> {
    type Output = Option<Cow<'a, str>>;
    fn to_cow(self) -> Self::Output {
        self.and_then(|s| Some(s.to_cow()))
    }
}
impl<'a> ToCow<'a> for Option<String> {
    type Output = Option<Cow<'a, str>>;
    fn to_cow(self) -> Self::Output {
        self.and_then(|s| Some(s.to_cow()))
    }
}
impl Help {
    pub fn new(cmps: Vec<cmp::ArcComponent>) -> Help {
        Help { components: cmps }
    }
    pub async fn send_help_list(_ctx: &cmp::Context, msg: &cmp::Message, title: &str, list: Vec<(&str, &str)>) -> serenity::Result<()> {
        match msg.channel_id.send_message(&_ctx.http, |m|
            m.embed(|embed| {
                embed.title("Aide");
                for (k,v) in list {
                    embed.field(k, v, false);
                }
                embed
            })
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    async fn r_command(&mut self, _: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        // if msg.content[1..].starts_with("help") {
        //     let list_words = msg.content.split(' ').skip(1).filter(|s| !s.is_empty());
            
        // } else {
        //     cmp::CommandMatch::NotMatched
        // }
        cmp::CommandMatch::NotMatched
    }
    async fn help_components<'a, 'b>(&'a self, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo<'a>, ()> {
        let mut components = Vec::new();
        for cmp in &self.components {
            components.push(cmp.lock().await.name().to_string());
        }
        match list_words.next() {
            Some(name) => {
                for cmp in &self.components {
                    let cmp = cmp.lock().;
                    let cmp = cmp.deref();
                    match (cmp.name(), cmp.group_parser()) {
                        (n, Some(grp)) if n == name => return Self::help_group(grp, list_words),
                        _ => ()
                    }
                }
                Err(())
            },
            None => Ok(HelpInfo{
                name: "Liste des composants".to_cow(),
                components: Some(components),
                .. Default::default()
            }),
        }
    }
    #[inline]
    fn help_node<'a, 'b>(node: &'a cmd::Node, name: &str, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo<'a>, ()> {
        if let Some(found) = node.groups.list().find(|g| g.name() == name) {
            Self::help_group(found, list_words)
        } else if let Some(found) = node.commands.list().find(|c| c.name() == name) {
            Self::help_command(found, list_words)
        } else {
            Err(())
        }
    }
    
    fn help_group<'a, 'b>(group: &'a cmd::Group, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo<'a>, ()> {
        match list_words.next() {
            Some(name) => Self::help_node(group.node(), name, list_words),
            None => {
                let mut groups = Vec::new();
                for grp in group.node().groups.list() {
                    groups.push((grp.name().to_cow(), grp.help().to_cow()));
                }
                let mut cmds = Vec::new();
                for cmd in group.node().commands.list() {
                    cmds.push((cmd.name().to_cow(), cmd.help().to_cow()));
                }
                Ok(HelpInfo{
                    name: Cow::Owned(format!("{} (Groupe de commande)", group.name())),
                    desc: group.help().to_cow(),
                    groups: if groups.is_empty() {None} else {Some(groups)},
                    commands: if cmds.is_empty() {None} else {Some(cmds)},
                    .. Default::default()
                })
            },
        }
    }
    fn help_command<'a, 'b>(command: &'a cmd::Command, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo<'a>, ()> {
        match list_words.next() {
            Some(_) => Err(()),
            None => {
                let mut params = Vec::new();
                for param in &command.params {
                    let name = match param.value_type() {
                        Some(vt) => format!("{} <{}>", param.name(), vt),
                        None => param.name().to_string(),
                    };
                    params.push((name, param.help().to_cow()));
                }
                Ok(HelpInfo{
                    name: format!("{} (Commande)", command.name()).to_cow(),
                    desc: command.help().to_cow(),
                    .. Default::default()
                })
            },
        }
    }
}