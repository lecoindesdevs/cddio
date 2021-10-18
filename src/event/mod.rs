mod sub_event_handler;

use std::{collections::HashMap, sync::{Arc}};

use futures::lock::Mutex;
use serenity::{async_trait, client::{Context, EventHandler}, model::{Permissions, prelude::Ready}};
use sub_event_handler::SubEventHandler;


#[derive(PartialEq, Eq, Hash, Clone)]
enum EventType {
    OnCacheReady,
    OnChannelCreate,
    OnCategoryCreate,
    OnCategoryDelete,
    OnChannelDelete,
    OnChannelPinsUpdate,
    OnChannelUpdate,
    OnGuildBanAddition,
    OnGuildBanRemoval,
    OnGuildCreate,
    OnGuildDelete,
    OnGuildEmojisUpdate,
    OnGuildIntegrationsUpdate,
    OnGuildMemberAddition,
    OnGuildMemberRemoval,
    OnGuildMemberUpdate,
    OnGuildMembersChunk,
    OnGuildRoleCreate,
    OnGuildRoleDelete,
    OnGuildRoleUpdate,
    OnGuildUnavailable,
    OnGuildUpdate,
    OnInviteCreate,
    OnInviteDelete,
    OnMessage,
    OnMessageDelete,
    OnMessageDeleteBulk,
    OnMessageUpdate,
    OnReactionAdd,
    OnReactionRemove,
    OnReactionRemoveAll,
    OnPresenceReplace,
    OnPresenceUpdate,
    OnReady,
    OnResume,
    OnShardStageUpdate,
    OnTypingStart,
    OnUnknown,
    OnUserUpdate,
    OnVoiceServerUpdate,
    OnVoiceStateUpdate,
    OnWebhookUpdate,
    OnInteractionCreate,
    OnIntegrationCreate,
    OnIntegrationUpdate,
    OnIntegrationDelete,
    OnApplicationCommandCreate,
    OnApplicationCommandUpdate,
    OnApplicationCommandDelete,
    OnStageInstanceCreate,
    OnStageInstanceUpdate,
    OnStageInstanceDelete,
    OnThreadCreate,
    OnThreadUpdate,
    OnThreadDelete,
    OnThreadListSync,
    OnThreadMemberUpdate,
    OnThreadMembersUpdate,
}

struct EventListener {
    name: String,
    listener: Mutex<Box<dyn SubEventHandler>>,
}

#[derive(Default)]
pub struct EventListenerContainer {
    event_listeners: Vec<EventListener>,
    // event_helper: HashMap<EventType, Vec<Arc<dyn EventHandler>>>
}

impl EventListenerContainer {
    pub fn init() -> EventListenerContainer {
        use EventType::*;
        let mut evts = EventListenerContainer::default();
        evts.register_event_listener("bot_start", Box::new(BotStart), vec![OnReady]);
        evts
    }
    fn register_event_listener(&mut self, name: &str, event_listener: Box<dyn SubEventHandler>, _:Vec<EventType>) {
        self.event_listeners.push(EventListener {
            name: name.to_string(),
            listener: Mutex::new(event_listener),
        });
    }
}

macro_rules! declare_event {
    (struct $name:ident { $(fn $name_fct:tt $def_struct:tt;)* }) => {
        
        impl EventHandler for $name {
            $(
                declare_event!{fn $name_fct $def_struct}
            )*
        }
    };
    (fn $name:ident ($($arg:ident: $type:ty),*)) => {
        async fn  $name(&self, ctx: Context, $($arg: $type),*) {
            let ctx = Mutex::new(ctx);
            $(
                let $arg = Mutex::new($arg);
            )*
            for evt in &self.event_listeners {
                let mut evt = evt.listener.lock().await;
                evt.as_mut().ready(&ctx, $(&$arg),*).await;
            }
        }
    };
}
#[async_trait]
impl EventHandler for EventListenerContainer {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.m_ready(ctx, ready).await
    }
}
// declare_event!{
//     struct EventListenerContainer {
//         fn ready(ready: Ready);
//     }
// }

impl EventListenerContainer {
    async fn m_ready(&self, ctx: Context, ready: Ready) {
        let ctx = Mutex::new(ctx);
        let ready = Mutex::new(ready);
        for evt in &self.event_listeners {
            let mut evt = evt.listener.lock().await;
            evt.as_mut().ready(&ctx, &ready).await
        }
    }
}

struct BotStart;

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
