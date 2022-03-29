use std::collections::HashMap;
use std::sync::Arc;

use crate::component_system::Component;
use crate::component_system as cmp;
use futures_locks::RwLock;
use serenity::model::id::GuildId;
use serenity::{model::{*, prelude::*}, async_trait};
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

#[async_trait]
impl Component for Autobahn {
    fn name(&self) -> &'static str {
        "autobahn"
    }
    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        use event::{*, Event::*};
        match evt {
            MessageCreate(MessageCreateEvent{message, ..}) => self.on_message_create(ctx, message).await,
            _ => Ok(())
        }
    }
}

impl Autobahn {
    pub fn new(cmp_moderation: Arc<Moderation>) -> Autobahn {
        Autobahn {
            sent_messages: RwLock::new(Vec::new()),
            cmp_moderation,
            max_messages: 4,
            max_time: chrono::Duration::seconds(20),
            mute_time: chrono::Duration::days(1),
        }
    }
    async fn on_message_create(&self, ctx: &cmp::Context, msg: &channel::Message) -> Result<(), String> {
        let guild_id = match msg.guild_id{
            Some(id) => id,
            None => return Ok(())
        };
        let msg_hash = hashers::fx_hash::fxhash64(msg.content.as_bytes());
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
            self.mute(ctx, guild_id, msg.author.id).await?;
            self.delete_messages(ctx, |(_, msg)| msg.who == msg_info.who).await;
            self.retain_messages(|(_,msg)| !(msg.who == msg_info.who)).await;
            return Ok(());
        } else {
            self.sent_messages.write().await.push((msg_hash, msg_info));
        } 
        
        Ok(())
    }
    async fn mute(&self, ctx: &cmp::Context, guild_id: GuildId, user_id: UserId) -> Result<(), String> {
        let res = self.cmp_moderation.mute(ctx, guild_id, user_id, Some("Suspicion de spam".to_string()), Some(self.mute_time)).await;
        match res {
            Ok(_) => Ok(()),
            Err(why) => Err(format!("Erreur autobahn mute : {}", why))
        }
    }
    async fn delete_messages<F>(&self, ctx: &cmp::Context, filter: F)
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
            match channel.delete_messages(ctx, &msgs).await {
                Ok(_) => (),
                Err(e) => println!("autobahn: Failed to delete messages: {}", e)
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
        let now = chrono::Utc::now();
        self.retain_messages(|(_,v)| now-v.time < self.max_time).await;
    }
}