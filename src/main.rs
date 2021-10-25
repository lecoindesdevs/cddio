mod bot;
mod component;
mod config;
#[macro_use]
mod util;


trait ResultLog {
    type OkType;
    fn expect_log(self, msg: &str) -> Self::OkType;
}
impl<T, S: AsRef<str>> ResultLog for Result<T, S> {
    type OkType=T;
    fn expect_log(self, msg: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) if msg.is_empty() => panic!("{}", e.as_ref()),
            Err(e) => panic!("{}: {}", msg, e.as_ref()),
        } 
    }
}
#[cfg(test)]
mod tests {
    use crate::component::{self, command_parser::{ParseError, matching}};

    #[test]
    fn command_parser() {
        use component::command_parser as cmd;
        
        let cmd = cmd::Command::new("command")
            .set_help("Une commande de test")
            .add_param(
                cmd::CommandParameter::new("param")
                    .set_help("Un paramètre")
                    .set_value_type("texte")
            );
        use component::command_parser::{split_shell, ParseError, matching};
        assert_eq!(cmd.try_match(&split_shell(r#"command -param "Je suis un parametre" -unknown"#)), Err(ParseError::UnknownParameter("-unknown")));
        assert_eq!(cmd.try_match(&split_shell(r#"command -param"#)), Err(ParseError::MissingParameterValue("-param")));
        assert_eq!(cmd.try_match(&split_shell(r#"command -param "Je suis un parametre""#)), Ok(matching::Command{
            path: vdq!["command"],
            params: vec![ matching::Parameter{ name: "param", value: "Je suis un parametre" } ]
        }));
    }
    #[test]
    fn group_parser() {
        use component::command_parser as cmd;
        let group = cmd::Group::new("group1")
            .add_group(
                cmd::Group::new("group2")
                    .add_command(cmd::Command::new("command1"))
                    .add_command(cmd::Command::new("command3")
                        .add_param(cmd::CommandParameter::new("param"))
                        .add_param(cmd::CommandParameter::new("param2")
                            .set_required(true)
                        )
                    )
            )
            .add_command(cmd::Command::new("command2"));
        use component::command_parser::split_shell;
        assert_eq!(group.try_match(&split_shell(r#"group1 group2 command1"#)), Ok(cmd::matching::Command {
            path: vdq!["group1", "group2", "command1"],
            params: vec![]
        }));
        assert_eq!(group.try_match(&split_shell(r#"group1 group2 command3 -param "Test param" -param2 test"#)), Ok(cmd::matching::Command {
            path: vdq!["group1", "group2", "command3"],
            params: vec![cmd::matching::Parameter{
                name:"param",
                value: "Test param"
            },
            cmd::matching::Parameter{
                name:"param2",
                value: "test"
            }]
        }));
        assert_eq!(group.try_match(&split_shell(r#"group1 group2 command3 -param "Test param""#)), Err(ParseError::RequiredParameters("param2".to_string())));
        assert_eq!(group.try_match(&split_shell(r#"group1 command2"#)), Ok(cmd::matching::Command {
            path: vdq!["group1", "command2"],
            params: vec![]
        }));
    }
    #[test]
    fn split_shell() {
        let args = component::command_parser::split_shell(r#"command -param "Je suis un parametre""#);
        assert_eq!(args, vec!["command", "-param", "Je suis un parametre"]);
    }
}

#[tokio::main]
async fn main() {
    let config = config::Config::read_file("./config.json").expect_log("Could not load the configuration file");
    let mut bot = bot::Bot::new(&config).await.or_else(|e|Err(e.to_string())).expect_log("");
    bot.start().await.or_else(|e|Err(e.to_string())).expect_log("Client won't start");
}
