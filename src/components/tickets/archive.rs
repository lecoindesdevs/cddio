use serenity::{
    client::Context, 
    model::{
        id::ChannelId,
        channel::Channel, prelude::Member,
    }
};
mod intern {
    use crate::log_warn;
    use std::collections::HashSet;

    use futures::StreamExt;
    use serde::Serialize;
    use serenity::client::Context;
    mod ser {
        pub use serenity::{
            model::{
                channel::{GuildChannel, Message},
                user::User,
                prelude::Member,
            },
        };
    }

    #[derive(Serialize, PartialEq, Eq, Hash)]
    pub struct ArchiveUser {
        pub id: u64,
        pub name: String,
        pub avatar: String,
    }
    impl From<&ser::User> for ArchiveUser {
        fn from(user: &ser::User) -> Self {
            Self {
                id: user.id.0,
                avatar: user.avatar_url().unwrap_or("".to_string()),
                name: format!("{}#{}", user.name, user.discriminator),
            }
        }
    }
    #[derive(Serialize, PartialEq, Eq, Hash)]
    pub struct ArchiveMember {
        pub user: ArchiveUser,
        pub guild_pseudo: String,
        pub avatar: String,
    }
    impl From<&ser::Member> for ArchiveMember {
        fn from(member: &ser::Member) -> Self {
            Self {
                user: ArchiveUser::from(&member.user),
                guild_pseudo: member.display_name().to_string(),
                avatar: member.user.avatar_url().unwrap_or("".to_string()),
            }
        }
    }
    #[derive(Serialize)]
    pub struct ArchiveMessage {
        pub id: u64,
        pub user_id: u64,
        pub content: String,
        pub attachments: Vec<String>,
        pub in_reply_to: Option<u64>,
        pub timestamp: i64,
    }
    impl From<ser::Message> for ArchiveMessage {
        fn from(message: ser::Message) -> Self {
            Self {
                id: message.id.0,
                user_id: message.author.id.0,
                content: message.content,
                attachments: message.attachments.iter().map(|a| a.url.clone()).collect(),
                in_reply_to: message.referenced_message.map(|m| m.id.0),
                timestamp: message.timestamp.unix_timestamp(),
            }
        }
    }
    #[derive(Serialize)]
    pub struct ArchiveChannel {
        pub id: u64,
        pub name: String,
        pub users: Vec<ArchiveUser>,
        pub messages: Vec<ArchiveMessage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub closed_by: Option<ArchiveMember>,
    }
    impl ArchiveChannel {
        pub async fn from_channel(ctx: &Context, channel: ser::GuildChannel, member: Option<&ser::Member>) -> Self {
            let mut users = HashSet::new();
            let mut messages = Vec::new();
            let mut msg_discord = channel.id.messages_iter(ctx).boxed();
            while let Some(message) = msg_discord.next().await {
                match message {
                    Ok(message) => {
                        users.insert(ArchiveUser::from(&message.author));
                        messages.push(ArchiveMessage::from(message));
                    },
                    Err(e) => log_warn!("Error getting message while archiving channel: {}", e)
                }
            }
            Self {
                id: channel.id.0,
                name: channel.name.clone(),
                users: users.into_iter().collect(),
                messages: messages,
                closed_by: member.map(|m| ArchiveMember::from(m)),
            }
        }
    }
}

pub async fn archive_ticket(ctx: &Context, channel: ChannelId, member: Option<&Member>) -> serenity::Result<()> {
    const ARCHIVE_PATH: &str = "./data/tickets/archives";

    let channel = match channel.to_channel(ctx).await? {
        Channel::Guild(channel) => channel,
        _ => unreachable!()
    };
    let name = channel.name.clone();
    let id = channel.id.0;
    let archive = serde_json::to_string(&intern::ArchiveChannel::from_channel(ctx, channel, member).await).unwrap();
    let path = format!("{}/{}-{}.json", ARCHIVE_PATH, id, name);
    async_std::fs::write(path, archive).await?;

    Ok(())
}