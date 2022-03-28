use std::collections::{HashSet, HashMap};
use std::sync::Arc;

use crate::component_system::Component;
use crate::component_system::{self as cmp, command_parser as cmd};
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
    id: MessageId
}

struct Autobahn {
    sent_messages: RwLock<HashMap<MessageHash, MessageInfo>>,
    cmp_moderation: Arc<Moderation>
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
            sent_messages: RwLock::new(HashMap::new()),
            cmp_moderation
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
            id: msg.id
        };
        {
            self.update_sent_messages();
            let nb_found = self.sent_messages.read().await.iter()
                .filter(|(k,v)| k == &&msg_hash && v.who == msg_info.who)
                .count();
            if nb_found > 4 {
                self.mute(ctx, guild_id, msg.author.id).await?;
                return Ok(());
            }
        }
        self.sent_messages.write().await.insert(msg_hash, msg_info);
        Ok(())
    }
    async fn mute(&self, ctx: &cmp::Context, guild_id: GuildId, user_id: UserId) -> Result<(), String> {
        let res = self.cmp_moderation.mute(ctx, guild_id, user_id, Some("Suspicion de spam".to_string()), Some(chrono::Duration::hours(24))).await;
        match res {
            Ok(_) => Ok(()),
            Err(why) => Err(format!("Erreur autobahn mute : {}", why))
        }
    }
    async fn update_sent_messages(&self) {
        let now = chrono::Utc::now();
        let mut sent_messages = self.sent_messages.write().await;
        *sent_messages = sent_messages.drain()
            .filter(|(_,v)| now-v.time < chrono::Duration::seconds(20))
            .collect();
    }
}