use std::{collections::HashMap, sync::Arc};

use serenity::{async_trait, client::{Context, EventHandler}, model::{Permissions, prelude::Ready}};


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
#[derive(Default)]
pub struct EventListener {
    event_listeners: HashMap<String, Arc<dyn EventHandler>>,
    event_helper: HashMap<EventType, Vec<Arc<dyn EventHandler>>>
}

impl EventListener {
    pub fn init() -> EventListener {
        use EventType::*;
        let mut evts = EventListener::default();
        evts.register_event("name", Arc::new(BotStart), &[OnReady]);
        evts
        
    }
    fn register_event(&mut self, name: &str, event_listener: Arc<dyn EventHandler>, event_types:&[EventType]) {
        // let arc_event = Arc::new(event_listener);
        self.event_listeners.insert(name.to_string(), Arc::clone(&event_listener));
        for evt in event_types {
            let vec_events = match self.event_helper.get_mut(evt) {
                Some(v) => v,
                None => {
                    self.event_helper.insert(evt.clone(),Vec::new());
                    self.event_helper.get_mut(evt).unwrap()
                }
            };
            vec_events.push(Arc::clone(&event_listener));
        }
    }
}

#[async_trait]
impl EventHandler for EventListener {
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(evt_list) = self.event_helper.get_mut(&EventType::OnReady) {
            for evt in evt_list {
                let evt = match Arc::get_mut(evt) {
                    Some(v) => v,
                    None => continue
                };
                evt.ready(ctx, ready);
            }

        }
    }
}

struct BotStart;

#[async_trait]
impl EventHandler for BotStart {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let http = Arc::clone(&ctx.http);
        match ready.user.invite_url(http, Permissions::empty()).await {
            Ok(v) => println!("Invitation: {}", v),
            Err(e) => println!("Unable to create invitation link: {}", e.to_string()),
        }
    }
}
