pub use serenity::client::Context;
pub use serenity::model::channel::Message;
use serenity::async_trait;

use super::ArcComponent;
use super::manager::{ArcManager, Manager};

/// Configuration du framework.
pub struct FrameworkConfig {
    pub prefix: char
}

/// Command handler qui dispatche les commandes aux composants.
/// 
/// Les commandes sont envoyées à chaque composant jusqu'à ce que le composant reconnaisse la commande.
pub struct Framework {
    components: ArcManager,
    config: FrameworkConfig
}

impl Framework {
    pub fn new(prefix: char, cmp_manager: ArcManager) -> Framework {
        Framework{
            components: cmp_manager,
            config: FrameworkConfig{ prefix }
        }
    }
    /// Retourne la configuration du framework.
    pub fn config(&self) -> &FrameworkConfig {
        &self.config
    }
}

#[async_trait]
impl serenity::framework::Framework for Framework {
    /// Dispatch les commandes aux composants.
    /// Le premier composant qui reconnait la commande est utilisé puis termine la fonction.
    async fn dispatch(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with(self.config.prefix) {
            return;
        }
        
        for mid in self.components.read().await.get_components() {
            let mut mid = mid.read().await;
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
