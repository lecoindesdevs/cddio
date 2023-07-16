use serde::Deserialize;


#[derive(Deserialize, PartialEq, Eq, Hash)]
pub struct ArchiveUser {
    pub id: u64,
    pub name: String,
    pub avatar: String,
}
#[derive(Deserialize, PartialEq, Eq, Hash)]
pub struct ArchiveMember {
    pub user: ArchiveUser,
    pub guild_pseudo: String,
    pub avatar: String,
}
#[derive(Deserialize, PartialEq, Eq, Hash)]
pub enum ArchiveReactionType {
    Custom {
        animated: bool,
        id: u64,
        name: Option<String>,
    },
    Unicode(String),
}
#[derive(Deserialize, PartialEq, Eq, Hash)]
pub struct ArchiveReaction {
    count: u64,
    emoji: ArchiveReactionType
}
#[derive(Deserialize)]
pub struct ArchiveMessage {
    pub id: u64,
    pub user_id: u64,
    pub content: String,
    pub attachments: Vec<String>,
    pub in_reply_to: Option<u64>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<ArchiveReaction>,
}
#[derive(Deserialize)]
pub struct ArchiveChannel {
    pub id: u64,
    pub name: String,
    pub users: Vec<ArchiveUser>,
    pub messages: Vec<ArchiveMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_by: Option<ArchiveMember>,
}

#[derive(Deserialize, Debug)]
struct DataTickets {
    /// Identifiants du channel et du message pour choisir le type de ticket
    /// Ces identifiants est enregistré pour pouvoir le remplacer si nécessaire
    msg_choose: Option<(u64, u64)>,
    /// [Catégories] de tickets
    /// 
    /// [Catégories]: CategoryTicket
    categories: Vec<Category>,
}