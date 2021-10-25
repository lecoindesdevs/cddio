pub use serenity::client::Context;
pub use serenity::model::channel::Message;
use serenity::framework::Framework;
use serenity::async_trait;

use super::ArcComponent;

pub struct FrameworkConfig {
    pub prefix: char
}
pub struct CDDFramework {
    // node: Node,
    components: Vec<ArcComponent>,
    config: FrameworkConfig
}

impl CDDFramework {
    pub fn new(prefix: char) -> CDDFramework {
        CDDFramework{
            components: Vec::new(),
            config: FrameworkConfig{ prefix }
        }
    }
    pub fn config(&self) -> &FrameworkConfig {
        &self.config
    }
    pub fn config_mut(&mut self) -> &mut FrameworkConfig {
        &mut self.config
    }
    pub fn add_component(&mut self, mid: ArcComponent) {
        self.components.push(mid);
    }
}

#[async_trait]
impl Framework for CDDFramework {
    async fn dispatch(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with(self.config.prefix) {
            return;
        }
        
        for mid in &self.components {
            let mut mid = mid.lock().await;
            if match mid.command(self.config(), &ctx, &msg).await {
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
                return;
            }
        }
    }
}
