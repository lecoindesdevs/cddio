use futures::lock::Mutex;
use serenity::{async_trait, client::{Context, EventHandler}, model::{Permissions, prelude::Ready}};
use super::SubEventHandler;

pub struct BotStart;

#[async_trait]
impl SubEventHandler for BotStart {
    async fn ready(&mut self, ctx: &Mutex<Context>, ready: &Mutex<Ready>) {
        let (username, invite) = { 
            let ctx = ctx.lock().await;
            let ready = ready.lock().await;
            (ready.user.name.clone(), ready.user.invite_url(&ctx.http, Permissions::empty()).await)
        };
        println!("{} is connected!", username);
        match invite {
            Ok(v) => println!("Invitation: {}", v),
            Err(e) => println!("Unable to create invitation link: {}", e.to_string()),
        }
    }
}
