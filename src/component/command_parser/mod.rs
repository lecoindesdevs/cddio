use std::{borrow::Cow, collections::HashMap};


pub mod matching {
    #[derive(Debug, PartialEq)]
    pub struct Parameter<'a> {
        pub name: &'a str,
        pub value: &'a str,
    }
    #[derive(Debug, PartialEq)]
    pub struct Command<'a> {
        pub name: &'a str,
        pub params: Vec<Parameter<'a>>,
    }
}
#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    NotMatched,
    UnknownParameter(&'a str),
    MissingParameterValue(&'a str),
    Todo
}
pub fn split_shell<'a>(txt: &'a str) -> Vec<&'a str> {
    let mut mode=false;
    txt.split(|c| {
        match (mode, c) {
            (_, '\"') => {
                mode = !mode;
                true
            }
            (false, ' ') => true,
            _ => false
        }
    })
    .filter(|s| !s.is_empty())
    .collect()
}

pub type ID = u32;
#[derive(Debug, Clone)]
pub struct CommandParameter {
    pub name: String,
    pub help: Option<String>,
    pub value_type: Option<String>
}
impl CommandParameter {
    pub fn new<S: Into<String>>(name: S) -> CommandParameter {
        CommandParameter {
            name: name.into(),
            help: None,
            value_type: None
        }
    }
    pub fn set_help<S: Into<String>>(mut self, h: S) -> CommandParameter {
        self.help = Some(h.into());
        self
    }
    pub fn help(&self) -> String {
        let mut msg = self.name.clone();
        if let Some(value_type) = &self.value_type {
            msg=format!("{} <{}>", msg, value_type);
        }
        if let Some(help) = &self.help {
            msg=format!("{}: {}", msg, help);
        }
        msg
    }
    pub fn value_type<S: Into<String>>(mut self, vt: S) -> CommandParameter {
        self.value_type = Some(vt.into());
        self
    }
}
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub help: Option<String>,
    pub params: Vec<CommandParameter>
}
impl Command {
    pub fn new<S: Into<String>>(name: S) -> Command {
        Command {
            name: name.into(),
            help: None,
            params: Vec::new()
        }
    }
    pub fn set_help<S: Into<String>>(mut self, h: S) -> Command {
        self.help = Some(h.into());
        self
    }
    pub fn help(&self) -> String {
        let mut msg = self.name.clone();
        if let Some(help) = &self.help {
            msg=format!("{}: {}", msg, help);
        }
        
        if !self.params.is_empty() {
            msg=format!("{}\nParamètres\n", msg);
            for param in &self.params {
                msg=format!("{}{}\n", msg, param.help());
            }
            msg.pop();
        }
        msg
    }

    pub fn add_param(mut self, param: CommandParameter) -> Command {
        self.params.push(param);
        self
    }

    pub fn try_match<'a>(&self, args: Vec<&'a str>) -> Result<matching::Command<'a>, ParseError<'a>> {
        if args.is_empty() {
            return Err(ParseError::Todo);
        }
        if args[0] != self.name {
            return Err(ParseError::NotMatched);
        }
        let mut params = Vec::new();
        let mut iter_args = args.iter().skip(1);
        while let Some(name) = iter_args.next() {
            if let None = self.params.iter().find(|cmdp| cmdp.name == name[1..]) {
                return Err(ParseError::UnknownParameter(name));
            }
            match iter_args.next() {
                Some(value) => params.push(matching::Parameter{name: &name[1..],value}),
                None => return Err(ParseError::MissingParameterValue(name))
            }
        }
        Ok(matching::Command{
            name: args[0],
            params,
        })
    }
}
#[derive(Debug, Clone)]
pub struct Group {
    name: String,
    help: Option<String>,
    node: Node
}
#[derive(Debug, Clone)]
struct Node {
    pub commands: Container<Command>,
    pub groups: Container<Group>,
}
#[derive(Debug, Clone)]
struct Container<T>(Option<HashMap<ID, T>>, ID);

impl<T> Container<T> {
    pub fn new() -> Self {
        Self(None, 1)
    }
    pub fn add(&mut self, value: T) -> ID {
        if let None = self.0 {
            self.0 = Some(HashMap::new());
        };
        let current_id = self.1;
        self.0.as_mut().unwrap().insert(current_id, value);
        
        self.1+=1;
        current_id
    }
    pub fn remove(&mut self, id: ID) -> Option<T> {
        if let Some(table) = &mut self.0 {
            table.remove(&id)
        } else {
            None
        }
    }
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Self::new()
    }
}