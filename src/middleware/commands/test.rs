use serenity::prelude::*;
use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::channel::Message,
};

#[group]
#[prefix = "test"]
#[commands(ping)]
pub struct Test;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("ping command");
    msg.channel_id.say(&ctx.http, "pong!").await?;

    Ok(())
}
