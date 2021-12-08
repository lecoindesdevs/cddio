mod time;

use crate::component::{self as cmp, command_parser as cmd};
use serenity::async_trait;


struct Moderation {
    node: cmd::Node,
}

#[async_trait]
impl cmp::Component for Moderation {
    fn name(&self) -> &'static str {
        "mod"
    }

    async fn command(&self, fw_config: &cmp::FrameworkConfig, ctx: &cmp::Context, msg: &cmp::Message) -> cmp::CommandMatch {
        cmp::CommandMatch::NotMatched
    }

    async fn event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        self.r_event(ctx, evt).await
    }
    fn node(&self) -> Option<&cmd::Node> {
        Some(&self.node)
    }
}

impl Moderation {
    fn new() -> Moderation {
        let ban = cmd::Command::new("ban")
            .set_help("Bannir un membre. Temporaire si l'argument for est présent.")
            .add_param(cmd::Argument::new("user")
                .set_value_type(cmd::ValueType::User)
                .set_help("Le membre à bannir")
                .set_required(true)
            )
            .add_param(cmd::Argument::new("for")
                .set_value_type(cmd::ValueType::String)
                .set_help("Pendant combien de temps")
            );
        Moderation {
            node: cmd::Node::new(),
        }
    }
    async fn r_event(&self, ctx: &cmp::Context, evt: &cmp::Event) -> Result<(), String> {
        todo!()
    }
}