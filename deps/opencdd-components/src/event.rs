use serenity::{model::event::Event, client::Context};

pub trait ComponentEvent {
    fn event(&mut self, ctx: &Context, event: &Event);
}