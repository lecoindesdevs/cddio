//! Anti-spam system

use crate::{log_error, log_warn, log_info};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::RwLock;
use cddio_macros::component;
use serenity::{model::{*, prelude::*}, client::Context};
use std::hash::Hash;
use super::Moderation;
type MessageHash = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MessageInfo {
    time: chrono::DateTime<chrono::Utc>,
    who: (id::GuildId, id::UserId),
    id: (ChannelId, MessageId)
}

pub struct Autobahn {
    sent_messages: RwLock<Vec<(MessageHash, MessageInfo)>>,
    cmp_moderation: Arc<Moderation>,

    max_messages: usize,
    max_time: chrono::Duration,
    mute_time: chrono::Duration,
}
#[component]
impl Autobahn {
    #[event(MessageCreate)]
    async fn on_message_create(&self, ctx: &Context, msg_create: &MessageCreateEvent) {
        let msg = &msg_create.message;
        if msg.author.id == ctx.cache.current_user().id {
            return;
        }
        log_info!("MessageCreateEvent");
        let msg_content = &msg.content;
        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => {
                log_info!("Message is not in a guild");
                return;
            },
        };
        let msg_hash = hashers::fx_hash::fxhash64(msg_content.as_bytes());
        log_info!("{} sent message, hash: {}", msg.author.name, msg_hash);

        let msg_info = MessageInfo {
            time: chrono::Utc::now(),
            who: (guild_id, msg.author.id),
            id: (msg.channel_id, msg.id)
        };
        
        self.remove_old_messages().await;
        
        let nb_found = self.sent_messages.read().await.iter()
            .filter(|(k,v)| k == &msg_hash && v.who == msg_info.who)
            .count()+1;
        if nb_found > self.max_messages {
            match msg.delete(ctx).await {
                Ok(_) => (),
                Err(e) => println!("autobahn: Failed to delete messages: {}", e)
            }
            if let Err(e) = self.cmp_moderation.mute(ctx, guild_id, msg.author.id, None, "Détection de spam".into(), Some(Utc::now() + self.mute_time)).await {
                log_error!("autobahn: Failed to mute user: {}", e);
                return;
            };
            self.delete_messages(ctx, |(_, msg)| msg.who == msg_info.who).await;
            self.retain_messages(|(_,msg)| !(msg.who == msg_info.who)).await;
        } else {
            self.sent_messages.write().await.push((msg_hash, msg_info));
        } 
    }
}

impl Autobahn {
    pub fn new(cmp_moderation: Arc<Moderation>) -> Autobahn {
        Autobahn {
            sent_messages: RwLock::new(Vec::with_capacity(100)),
            cmp_moderation,
            max_messages: 4,
            max_time: chrono::Duration::seconds(20),
            mute_time: chrono::Duration::days(1),
        }
    }
    async fn delete_messages<F>(&self, ctx: &Context, filter: F)
        where F: Fn(&&(MessageHash, MessageInfo)) -> bool
    {
        let mut msg_to_delete: HashMap<ChannelId, Vec<MessageId>> = HashMap::new();
        self.sent_messages.read().await.iter()
            .filter(filter)
            .for_each(|(_, msg)| {
                msg_to_delete.entry(msg.id.0)
                    .or_insert_with(Vec::new)
                    .push(msg.id.1);
            });
        for (channel, msgs) in msg_to_delete.into_iter() {
            println!("autobahn: Deleting {} messages from channel {}", msgs.len(), channel);
            match channel.delete_messages(ctx, &msgs).await {
                Ok(_) => (),
                Err(e) => log_warn!("autobahn: Failed to delete messages: {}", e)
            }
        }
    }
    #[inline]
    async fn retain_messages<F>(&self,filter: F)
        where F: Fn(&(MessageHash, MessageInfo)) -> bool
    {
        let mut sent_messages = self.sent_messages.write().await;
        sent_messages.retain(filter);
    }
    #[inline]
    async fn remove_old_messages(&self) {
        self.retain_messages(|(_,v)| Utc::now()-v.time < self.max_time).await;
    }
}