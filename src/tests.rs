use crate::component::command_parser::{self as cmd, ParseError, split_shell, matching};

#[test]
fn command_parser() {
    
    let cmd = cmd::Command::new("command")
        .set_help("Une commande de test")
        .add_param(
            cmd::CommandParameter::new("param")
                .set_help("Un paramètre")
                .set_value_type("texte")
        );
    assert_eq!(cmd.try_match(&split_shell(r#"command -param "Je suis un parametre" -unknown"#)), Err(ParseError::UnknownParameter("-unknown")));
    assert_eq!(cmd.try_match(&split_shell(r#"command -param"#)), Err(ParseError::MissingParameterValue("-param")));
    assert_eq!(cmd.try_match(&split_shell(r#"command -param "Je suis un parametre""#)), Ok(matching::Command{
        path: vdq!["command"],
        params: vec![ matching::Parameter{ name: "param", value: "Je suis un parametre" } ]
    }));
}
#[test]
fn group_parser() {
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
    
    let matched = group.try_match(&split_shell(r#"group1 group2 command1"#));
    assert_eq!(matched, Ok(cmd::matching::Command {
        path: vdq!["group1", "group2", "command1"],
        params: vec![]
    }));
    println!("{:?}", matched.unwrap().path.as_slices());
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
fn split_cmd() {
    let args = split_shell(r#"command -param "Je suis un parametre""#);
    assert_eq!(args, vec!["command", "-param", "Je suis un parametre"]);
}
