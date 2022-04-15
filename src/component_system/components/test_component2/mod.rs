use opencdd_macros::*;
use serenity::{model::event::Event, client::Context};
use serenity::model::id::{ChannelId, GuildId, UserId, RoleId};

trait Component2 {
    fn event(&mut self, ctx: &Context, event: &Event);
}

struct Test;

#[commands]
impl Test {
    #[command]
    fn ban(&self, qui: RoleId, pourquoi: String, pendant: Option<String>) {
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
