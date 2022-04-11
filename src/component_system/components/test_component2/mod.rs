use opencdd_macros::*;
use serenity::{model::event::Event, client::Context};

trait Component2 {
    fn event(&mut self, ctx: &Context, event: &Event);
}

struct Test;

#[commands]
impl Test {
    #[command]
    fn command_test(&self) {
        println!("test");
    }
    #[event(MessageCreate)]
    fn test2(&self) {
        println!("test2");
    }
}
