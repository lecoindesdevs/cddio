use serenity::{async_trait, utils::Colour};

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

    async fn event(&mut self, _: &cmp::Context, _: &cmp::Event) -> Result<(), String> {
        Ok(())
    }
}
#[derive(Debug, Default)]
struct HelpInfo {
    name: String,
    desc: Option<String>,
    permission: Option<String>,
    groups: Option<Vec<(String, Option<String>)>>,
    commands: Option<Vec<(String, Option<String>)>>,
    params: Option<Vec<(String, Option<String>)>>,
    components: Option<Vec<String>>
}

impl Help {
    pub fn new(cmps: Vec<cmp::ArcComponent>) -> Help {
        Help { components: cmps }
    }
    async fn send_error(_ctx: &cmp::Context, msg: &cmp::Message) -> serenity::Result<()> {
        match msg.channel_id.send_message(&_ctx.http, |m|
            m.embed(|embed| {
                embed
                    .title("Aide")
                    .description("Aucun groupe ni commande trouvé.")
                    .color(Colour::from_rgb(204, 0, 0))
            })
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    async fn send_help(_ctx: &cmp::Context, msg: &cmp::Message, info: HelpInfo) -> serenity::Result<()> {
        match msg.channel_id.send_message(&_ctx.http, |m|
            m.embed(|embed| {
                embed.title(format!("{} - Aide", info.name));
                if let Some(desc) = info.desc {
                    embed.description(desc);
                }
                let permission = info.permission;
                embed.footer(|f| f.text(format!("Autorisé par {}", if let Some(v) = &permission {format!("@{}", v.as_str())} else {"tout le monde".to_string()})));
                
                let mut make_field = |name: &str, groups: Option<Vec<(String, Option<String>)>>| 
                    if let Some(groups) = groups {
                        let mut value = String::new();
                        let plural = if groups.len()>1 {"s"} else {""};
                        for group in groups {
                            if let Some(desc) = group.1 {
                                value = format!("{}**{}** : {}\n", value, group.0, desc);
                            } else {
                                value = format!("{}**{}**\n", value, group.0);
                            }
                        }
                        value.pop();
                        embed.field(format!("{}{}", name, plural), value, false);
                    };
                make_field("Groupe", info.groups);
                make_field("Commande", info.commands);
                make_field("Paramètre", info.params);
                if let Some(cmps) = info.components {
                    let mut value = String::new();
                    let plural = if cmps.len()>1 {"s"} else {""};
                    for cmp in cmps {
                        value = format!("{}**{}**\n", value, cmp);
                    }
                    value.pop();
                    embed.field(format!("Composant{}", plural), value, false);
                };
                embed.color(Colour::from_rgb(0, 204, 0));
                embed
            })
        ).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    async fn r_command(&mut self, _: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        if msg.content[1..].starts_with("help") {
            let list_words = msg.content.split(' ').skip(1).filter(|s| !s.is_empty());
            let send_result = match self.help_components(list_words).await {
                Ok(v) => Self::send_help(ctx, msg, v).await,
                Err(_) => Self::send_error(ctx, msg).await
            };
            match send_result {
                Ok(_) => cmp::CommandMatch::Matched,
                Err(e) => cmp::CommandMatch::Error(e.to_string()),
            }
        } else {
            cmp::CommandMatch::NotMatched
        }
    }
    async fn help_components<'a, 'b>(&'a self, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo, ()> {
        let mut components = Vec::new();
        for cmp in &self.components {
            components.push(cmp.lock().await.name().to_string());
        }
        match list_words.next() {
            Some(name) => {
                for cmp in &self.components {
                    let cmp = cmp.lock().await;
                    match (cmp.name(), cmp.group_parser()) {
                        (n, Some(grp)) if n == name => return Self::help_group(grp, None, list_words),
                        _ => ()
                    }
                }
                Err(())
            },
            None => Ok(HelpInfo{
                name: "Liste des composants".to_string(),
                components: Some(components),
                .. Default::default()
            }),
        }
    }
    #[inline]
    fn help_node<'a, 'b>(node: &'a cmd::Node, permission: Option<&'a str>, name: &str, list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo, ()> {
        if let Some(found) = node.groups.list().find(|g| g.name() == name) {
            Self::help_group(found, permission, list_words)
        } else if let Some(found) = node.commands.list().find(|c| c.name() == name) {
            Self::help_command(found, permission, list_words)
        } else {
            Err(())
        }
    }
    
    fn help_group<'a, 'b>(group: &'a cmd::Group, permission: Option<&'a str>, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo, ()> {
        let permission = match group.permission() {
            Some(v) => Some(v),
            None => permission,
        };
        match list_words.next() {
            Some(name) => Self::help_node(group.node(), permission, name, list_words),
            None => {
                let mut groups = Vec::new();
                for grp in group.node().groups.list() {
                    groups.push((grp.name().to_string(), grp.help().and_then(|v| Some(v.to_string()))));
                }
                let mut cmds = Vec::new();
                for cmd in group.node().commands.list() {
                    cmds.push((cmd.name().to_string(), cmd.help().and_then(|v| Some(v.to_string()))));
                }
                Ok(HelpInfo{
                    name: format!("{} (Groupe de commande)", group.name()),
                    permission: group.permission().and_then(|v| Some(v.to_string())),
                    desc: group.help().and_then(|v| Some(v.to_string())),
                    groups: if groups.is_empty() {None} else {Some(groups)},
                    commands: if cmds.is_empty() {None} else {Some(cmds)},
                    .. Default::default()
                })
            },
        }
    }
    fn help_command<'a, 'b>(command: &'a cmd::Command, permission: Option<&'a str>, mut list_words: impl Iterator<Item = &'b str>) -> Result<HelpInfo, ()> {
        let permission = match command.permission() {
            Some(v) => Some(v),
            None => permission,
        };
        match list_words.next() {
            Some(_) => Err(()),
            None => {
                let mut params = Vec::new();
                for param in &command.params {
                    let name = match param.value_type() {
                        Some(vt) => format!("{} <{}>", param.name(), vt),
                        None => param.name().to_string(),
                    };
                    params.push((name, param.help().and_then(|v| Some(v.to_string()))));
                }
                Ok(HelpInfo{
                    name: format!("{} (Commande)", command.name()),
                    permission: permission.and_then(|v| Some(v.to_string())),
                    desc: command.help().and_then(|v| Some(v.to_string())),
                    .. Default::default()
                })
            },
        }
    }
}