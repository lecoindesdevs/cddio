pub use serenity::client::Context;
pub use serenity::model::channel::Message;
use serenity::framework::Framework;
use serenity::async_trait;

use super::ArcComponent;

// pub type ID = u32;

// pub trait CommandFunc {
//     fn func(&mut self);
// }

// pub struct Command {
//     name: Cow<'static, str>,
    
// }
// pub struct Group {
//     name: Cow<'static, str>,
//     node: Option<Node>
// }

// struct Node {
//     pub commands: Container<Command>,
//     pub groups: Container<Group>,
// }

// struct Container<T>(Option<HashMap<ID, T>>, ID);

// impl<T> Container<T> {
//     pub fn new() -> Self {
//         Self(None, 1)
//     }
//     pub fn add(&mut self, value: T) -> ID {
//         if let None = self.0 {
//             self.0 = Some(HashMap::new());
//         };
//         let current_id = self.1;
//         self.0.unwrap().insert(current_id, value);
        
//         self.1+=1;
//         current_id
//     }
//     pub fn remove(&mut self, id: ID) -> Option<T> {
//         if let Some(table) = self.0 {
//             table.remove(&id)
//         } else {
//             None
//         }
//     }
// }

// impl<T> Default for Container<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

pub struct CDDFramework {
    // node: Node,
    components: Vec<ArcComponent>,
    prefix: char
}

impl CDDFramework {
    pub fn new(prefix: char) -> CDDFramework {
        CDDFramework{
            components: Vec::new(),
            prefix
        }
    }
    pub fn add_component(&mut self, mid: ArcComponent) {
        self.components.push(mid);
    }
}

#[async_trait]
impl Framework for CDDFramework {
// impl CDDFramework {
    async fn dispatch(&self, ctx: Context, msg: Message) {
        'main: loop {
            if let Some(c) = msg.content.chars().next() {
                if c != self.prefix {
                    break 'main;
                }
            } else {
                break 'main;
            }
            for mid in &self.components {
                let mut mid = mid.lock().await;
                if match mid.command(&ctx, &msg).await {
                    super::CommandMatch::Matched => true,
                    super::CommandMatch::NotMatched => false,
                    super::CommandMatch::Error(what) => {
                        println!("[{}] Module {} command error: {}\nMessage: {:?}\n\n",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), 
                            mid.name(),
                            what,
                            msg
                        );
                        true
                    },
                } {
                    break 'main;
                }
            }
        }
    }
}
