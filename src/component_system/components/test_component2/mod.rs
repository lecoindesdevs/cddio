use opencdd_macros::*;
use serenity::{model::event::Event, client::Context};
use serenity::model::id::RoleId;

use super::utils::app_command::ApplicationCommandEmbed;

trait ComponentEvent {
    fn event(&mut self, ctx: &Context, event: &Event);
}
trait ComponentDeclarative {
    fn declarative(&self) -> &'static Node;
}

struct Node {
    commands: &'static [Command],
    children: &'static [Node],
}
struct Command {
    name: &'static str,
    description: &'static str,
    params: &'static [Parameter],
}
struct Parameter {
    name: &'static str,
    type_: &'static str,
    description: &'static str,
}


struct Test;

#[commands]
impl Test {
    #[command]
    fn ban(&self, ctx: &Context, appcmd: &ApplicationCommandEmbed, qui: RoleId, pourquoi: String, pendant: Option<String>) {
        println!("command ban");
        println!("{}", qui);
        println!("{}", pourquoi);
        println!("{:?}", pendant);
    }
    #[command]
    fn kick(&self, qui: RoleId, pourquoi: String) {
        println!("command kick");
        println!("{}", qui);
        println!("{}", pourquoi);
    }
    // #[event(MessageCreate)]
    // fn test2(&self) {
    //     println!("test2");
    // }
}
