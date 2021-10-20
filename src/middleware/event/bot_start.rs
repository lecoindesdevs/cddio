use futures::lock::Mutex;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::{Permissions, event::{Event, ReadyEvent}};
use super::SubRawEventHandler;

pub struct BotStart;

#[async_trait]
impl SubRawEventHandler for BotStart {
    async fn raw_event(&mut self, ctx: &Mutex<Context>, evt: &Mutex<Event>) {
        let evt = evt.lock().await.clone();
        if let Event::Ready(ReadyEvent{ready, ..}) = evt {
            let (username, invite) = { 
                let ctx = ctx.lock().await;
                (ready.user.name.clone(), ready.user.invite_url(&ctx.http, Permissions::empty()).await)
            };
            println!("{} is connected!", username);
            match invite {
                Ok(v) => println!("Invitation: {}", v),
                Err(e) => println!("Unable to create invitation link: {}", e.to_string()),
            }
        }
    }
}
