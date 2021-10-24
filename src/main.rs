mod bot;
mod component;
mod config;


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
    use crate::component;

    #[test]
    fn command() {
        use component::command_parser as cmd;
        let cmd = cmd::Command::new("command")
            .set_help("Une commande de test")
            .add_param(
                cmd::CommandParameter::new("param")
                    .set_help("Un paramètre")
                    .value_type("texte")
            );
        use component::command_parser::{split_shell, ParseError, matching};
        assert_eq!(cmd.try_match(split_shell(r#"command -param "Je suis un parametre" -unknown"#)), Err(ParseError::UnknownParameter("-unknown")));
        assert_eq!(cmd.try_match(split_shell(r#"command -param"#)), Err(ParseError::MissingParameterValue("-param")));
        assert_eq!(cmd.try_match(split_shell(r#"command -param "Je suis un parametre""#)), Ok(matching::Command{
            name: "command",
            params: vec![ matching::Parameter{ name: "param", value: "Je suis un parametre" } ]
        }));
    }
    #[test]
    fn split_shell() {
        let args = component::command_parser::split_shell(r#"command -param "Je suis un parametre""#);
        for v in args {
            println!("{}", v);
        }
    }
}

#[tokio::main]
async fn main() {
    let config = config::Config::read_file("./config.json").expect_log("Could not load the configuration file");
    let mut bot = bot::Bot::new(&config).await.or_else(|e|Err(e.to_string())).expect_log("");
    bot.start().await.or_else(|e|Err(e.to_string())).expect_log("Client won't start");
}
