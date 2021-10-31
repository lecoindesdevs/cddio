#![allow(dead_code)]
use std::collections::VecDeque;

pub mod matching {
    use std::collections::VecDeque;

    #[derive(Debug, PartialEq)]
    pub struct Parameter<'a> {
        pub name: &'a str,
        pub value: &'a str,
    }
    #[derive(Debug, PartialEq)]
    pub struct Command<'a> {
        pub path: VecDeque<&'a str>,
        pub params: Vec<Parameter<'a>>,
        pub permission: Option<&'a str>
    }
    impl<'a> Command<'a> {
        pub fn get_command(&self) -> &'a str {
            self.path.as_slices().1[0]
        }
        pub fn get_groups(&self) -> &[&'a str] {
            &self.path.as_slices().0
        }
    }
}

pub trait Named {
    fn name(&self) -> &str;
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    NotMatched,
    UnknownParameter(&'a str),
    MissingParameterValue(&'a str),
    ExpectedPath(&'a str),
    RequiredParameters(String),
    Todo
}
impl<'a> ToString for ParseError<'a> {
    fn to_string(&self) -> String {
        match &self {
            ParseError::NotMatched => "Command not found".to_string(),
            ParseError::UnknownParameter(v) => format!("Unknown parameter: {}", v),
            ParseError::MissingParameterValue(v) => format!("Missing value for parameter {}", v),
            ParseError::ExpectedPath(v) => format!("Expected group or command name after {}", v),
            ParseError::RequiredParameters(v) => format!("Parameter required: {}", v),
            ParseError::Todo => "Unknown parser error".to_string(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub help: Option<String>,
    pub value_type: Option<String>,
    pub required: bool
}
impl Named for Argument {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Argument {
    pub fn new<S: Into<String>>(name: S) -> Argument {
        Argument {
            name: name.into(),
            help: None,
            value_type: None,
            required: false
        }
    }
    
    pub fn set_help<S: Into<String>>(mut self, h: S) -> Argument {
        self.help = Some(h.into());
        self
    }
    pub fn help(&self) -> Option<&str> {
        match &self.help {
            Some(h) => Some(&h),
            None => None,
        }
    }
    pub fn set_value_type<S: Into<String>>(mut self, vt: S) -> Argument {
        self.value_type = Some(vt.into());
        self
    }
    pub fn value_type(&self) -> Option<&str> {
        match &self.value_type {
            Some(v) => Some(&v),
            None => None,
        }
    }
    pub fn set_required(mut self, req: bool) -> Argument {
        self.required = req;
        self
    }
}
#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub permission: Option<String>,
    pub help: Option<String>,
    pub params: Vec<Argument>
}
impl Named for Command {
    fn name(&self) -> &str {
        &self.name
    }
}
impl Command {
    pub fn new<S: Into<String>>(name: S) -> Command {
        Command {
            name: name.into(),
            permission: None,
            help: None,
            params: Vec::new()
        }
    }
    pub fn set_permission<S: Into<String>>(mut self, permission: S) -> Self {
        self.permission = Some(permission.into());
        self
    }
    pub fn permission(&self) -> Option<&str> {
        match &self.permission {
            Some(v) => Some(&v),
            None => None,
        }
    }
    pub fn set_help<S: Into<String>>(mut self, h: S) -> Command {
        self.help = Some(h.into());
        self
    }
    pub fn help(&self) -> Option<&str> {
        match &self.help {
            Some(h) => Some(&h),
            None => None,
        }
    }
    
    pub fn add_param(mut self, param: Argument) -> Command {
        self.params.push(param);
        self
    }

    pub fn try_match<'a>(&'a self, permission: Option<&'a str>, args: &[&'a str]) -> Result<matching::Command<'a>, ParseError<'a>> {
        if args.is_empty() {
            return Err(ParseError::Todo);
        }
        if args[0] != self.name {
            return Err(ParseError::NotMatched);
        }
        let permission = match &self.permission {
            Some(v) => Some(v.as_str()),
            None => permission,
        };
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
        let it_req = self.params.iter().filter(|p| p.required);
        let mut it_req_missing = it_req.filter(|p1| params.iter().find(|p2| p1.name == p2.name).is_none());
        if let Some(param_missing) = it_req_missing.next() {
            return Err(ParseError::RequiredParameters(param_missing.name.clone()));
        }
        Ok(matching::Command{
            path: {let mut v = VecDeque::new(); v.push_back(args[0]); v},
            permission,
            params,
        })
    }
}
#[derive(Debug, Clone)]
pub struct Group {
    name: String,
    permission: Option<String>,
    help: Option<String>,
    node: Node
}
impl Group {
    pub fn new<S: Into<String>>(name: S) -> Group {
        Group { 
            name: name.into(), 
            permission: None,
            help: None, 
            node: Node::new() 
        }
    }
    pub fn add_group(mut self, grp: Group) -> Group {
        self.node.groups.add(grp);
        self
    }
    pub fn add_command(mut self, cmd: Command) -> Group {
        self.node.commands.add(cmd);
        self
    }
    pub fn set_permission<S: Into<String>>(mut self, permission: S) -> Self {
        self.permission = Some(permission.into());
        self
    }
    pub fn permission(&self) -> Option<&str> {
        match &self.permission {
            Some(v) => Some(&v),
            None => None,
        }
    }
    pub fn set_help<S: Into<String>>(mut self, h: S) -> Group {
        self.help = Some(h.into());
        self
    }
    pub fn help(&self) -> Option<&str> {
        match &self.help {
            Some(h) => Some(&h),
            None => None,
        }
    }
    pub fn node(&self) -> &Node {
        &self.node
    }
    pub fn try_match<'a>(&'a self, permission: Option<&'a str>, args: &[&'a str]) -> Result<matching::Command<'a>, ParseError<'a>> {
        if args[0] != self.name {
            return Err(ParseError::NotMatched);
        }
        if args.len() == 1 {
            return Err(ParseError::ExpectedPath(args[0]))
        }
        if args[1].starts_with('-') {
            return Err(ParseError::ExpectedPath(args[0]));
        }
        let permission = match &self.permission {
            Some(v) => Some(v.as_str()),
            None => permission,
        };
        match self.node.commands.find(args[1]) {
            Some(cmd) => cmd.try_match(permission, &args[1..]),
            None => match self.node.groups.find(args[1]) {
                Some(grp) => grp.try_match(permission, &args[1..]),
                None => Err(ParseError::NotMatched),
            },
        }
        .and_then(|mut cmd| Ok({cmd.path.push_front(args[0]); cmd}))
    }
}
impl Named for Group {
    fn name(&self) -> &str {
        &self.name
    }
}
#[derive(Debug, Clone)]
pub struct Node {
    pub commands: Container<Command>,
    pub groups: Container<Group>,
}
impl Node {
    pub fn new() -> Node {
        Node { 
            commands: Container::new(), 
            groups: Container::new() 
        }
    }
}
#[derive(Debug, Clone)]
pub struct Container<T: Named>(Vec<T>);

impl<T: Named> Container<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn add(&mut self, value: T) {
        if let Some(_) = self.find(value.name()) {
            panic!("Container values MUST BE name distinct");
        }
        self.0.push(value);
    }
    pub fn find(&self, name: &str) -> Option<&T> {
        self.0.iter().find(|v| v.name() == name)
    }
    pub fn list(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
    pub fn remove(&mut self, name: &str)  {
        let id = self.0.iter().take_while(|v| v.name() == name).count();
        if id>=self.0.len() {
            panic!("Container remove: {} not found", name);
        }
        self.0.remove(id);
    }
}

impl<T: Named> Default for Container<T> {
    fn default() -> Self {
        Self::new()
    }
}