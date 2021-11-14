//! Parseur de commande
//! 
//! Ce module a été conçu pour faciliter la déclaration de composants, de groupes et de commandes.
//! 
//! Il permet de déclarer des commandes, de déclarer des groupes de commandes, des composants, des arguments au commande, 
//! de définir leur comportement tel que des arguments requis, leur type, leur valeur par défaut, leur valeur minimum et maximum, les role qui y sont permis, etc.
//! 
//! Il permet aussi l'affichage d'aide automatisé des groupes et des commandes par le biais de composant [`help`].
//! 
//! Le parseur de commande foncitonne de la même manière qu'un parseur de ligne de commande: 
//! - Chaque partie de la ligne de commande est séparée par des espaces, sauf les arguments quotés. `mot1 mot2 "mot 3"`
//! - Les groupes et les commandes sont analysés en mot clé au debut de la commande et de manière recursif : `group [...sous_groupes...] command`
//! - Les arguments avec des tirets tel que `-nom_parametre valeur` ou `-nom_parametre "valeur avec des espaces"` sont considérés comme des paramètres. (TODO : ajouter des flags)
//! - Les arguments sans tiret sont considérés comme des arguments de commande.
//! 
//! Le parseur est là pour aidé a concevoir une commande, mais ne fait pas de traitement.
//! 
//! # Exemple
//! 
//! ```rust
//! let misc_definition = cmd::Group::new("misc")
//!     .set_help("Commande diverse, sans catégorie, ou de test")
//!     .add_command(cmd::Command::new("concat")
//!         .set_help("Permet de concaténer deux chaînes de caractères")
//!         .add_argument(cmd::Argument::new("string1")
//!             .set_help("Première chaîne de caractères à concaténer")
//!             .set_required(true)
//!         )
//!         .add_argument(cmd::Argument::new("string2")
//!             .set_help("Deuxième chaîne de caractères à concaténer")
//!             .set_required(true)
//!         )
//! )
//! ```
//! **Utilisation**: `misc concat -string1 "Hello" -string2 " world"`
//! 
//! [`help`]: crate::component::components::help


#![allow(dead_code)]
use std::collections::VecDeque;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

/// Structures de retour d'une commande qui a match avec le parseur
pub mod matching {
    use std::collections::VecDeque;
    /// Information de paramètre de commande que le parseur a matché
    #[derive(Debug, PartialEq)]
    pub struct Parameter<'a> {
        pub name: &'a str,
        pub value: &'a str,
    }
    /// Information de commande que le parseur a matché
    #[derive(Debug, PartialEq)]
    pub struct Command<'a> {
        /// Chemin de la commande. Exemple : `["group", "subgroup", "command"]`
        pub path: VecDeque<&'a str>,
        /// Paramètres de la commande. Exemple avec la commande concat : `[Parameter { name: "string1", value: "Hello" }, Parameter { name: "string2", value: " world" }]`
        pub params: Vec<Parameter<'a>>,
        /// Role pouvant lancer la commande. Tout le monde si None.
        pub permission: Option<&'a str>,
        /// Arguments variadiques
        pub arguments: Vec<&'a str>,
    }
    impl<'a> Command<'a> {
        /// Retourne le nom de la commande. Exemple : `["group", "subgroup", "command"]` -> `command`
        pub fn get_command(&self) -> &'a str {
            self.path.as_slices().1[0]
        }
        /// Retourne la suite de groupe. Exemple : `["group", "subgroup", "command"]` -> `["group", "subgroup"]`
        pub fn get_groups(&self) -> &[&'a str] {
            &self.path.as_slices().0
        }

        pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
            self.params.iter().find(|p| p.name == name)
        }
    }
}

/// Trait d'objet nommé.
/// 
/// A la base, il devait forcer les groupes, les commandes et les composants à avoir un nom,
/// puis d'être utilisé lu dynamiquement. 
/// Mais au final, une approche static a été utilisé.
pub trait Named {
    fn name(&self) -> &str;
}
/// Erreur de parsing
#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    /// La commande n'a pas matché
    NotMatched,
    /// Un groupe a matché, mais pas la commande
    PartiallyNotMatched(&'a str),
    /// Le paramètre n'a est inconnu
    UnknownParameter(&'a str),
    /// La valeur du paramètre est absente
    MissingParameterValue(&'a str),
    /// Chemin attendu 
    ExpectedPath(&'a str),
    /// Paramètres requis manquants
    RequiredParameters(String),
    /// Erreur inconnue
    Todo
}
impl<'a> ToString for ParseError<'a> {
    fn to_string(&self) -> String {
        match &self {
            ParseError::NotMatched => "Commande inconnue".to_string(),
            ParseError::PartiallyNotMatched(v)=> format!("Groupe ou commande inconnu {}", v),
            ParseError::UnknownParameter(v) => format!("Paramètre {} inconnu", v),
            ParseError::MissingParameterValue(v) => format!("Valeur du paramètre {} manquant", v),
            ParseError::RequiredParameters(v) => format!("Paramètre {} requis", v),
            ParseError::ExpectedPath(v) => format!("Groupe ou commande attendu après {}", v),
            ParseError::Todo => "Unknown parser error".to_string(),
        }
    }
}
/// Convertit une chaine de caractère en groupe d'arguments 
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

///Argument de commande
#[derive(Debug, Clone)]
pub struct Argument {
    /// Nom de l'argument
    pub name: String,
    /// Description de l'argument
    pub help: Option<String>,
    /// Type de valeur
    pub value_type: ApplicationCommandOptionType,
    /// L'argument requis si vrai
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
            value_type: ApplicationCommandOptionType::String,
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
    pub fn set_value_type(mut self, vt: ApplicationCommandOptionType) -> Argument {
        match vt {
            ApplicationCommandOptionType::SubCommand => panic!("Commande non supporté pour les arguments, utilisez les commandes natives"),
            ApplicationCommandOptionType::SubCommandGroup => panic!("Groupe de commande non supporté pour les arguments, utilisez les groupes de commande natifs"),
            _ => self.value_type = vt
        }
        self
    }
    pub fn value_type(&self) -> ApplicationCommandOptionType {
        self.value_type
    }
    pub fn value_type_str(&self) -> &'static str {
        match self.value_type {
            ApplicationCommandOptionType::String => "string",
            ApplicationCommandOptionType::Integer => "integer",
            ApplicationCommandOptionType::Boolean => "boolean",
            ApplicationCommandOptionType::User => "user",
            ApplicationCommandOptionType::Channel => "channel",
            ApplicationCommandOptionType::Role => "role",
            ApplicationCommandOptionType::Mentionable => "mentionable",
            ApplicationCommandOptionType::Number => "number",
            _ => "unknown"
        }
    }
    pub fn set_required(mut self, req: bool) -> Argument {
        self.required = req;
        self
    }
    pub fn required(&self) -> bool {
        self.required
    }
}
#[derive(Debug, Clone)]
pub struct Command {
    /// Nom de la commande
    pub name: String,
    /// Type d'arguments variadique si besoin
    pub arguments: Option<String>,
    /// Description de la commande
    pub help: Option<String>,
    /// Role pouvant lancer la commande. Tout le monde si None.
    pub permission: Option<String>,
    /// Liste des arguments de la commande
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
            arguments: None,
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
    pub fn params(&self) -> &Vec<Argument> {
        &self.params
    }
    pub fn set_arguments(mut self, arg: String) -> Command {
        self.arguments = Some(arg);
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
        let mut arguments: Vec<&str> = Vec::new();
        while let Some(name) = iter_args.next() {
            if !name.starts_with('-') {
                if self.arguments.is_some() {
                    arguments.push(name);
                    continue;
                } else {
                    return Err(ParseError::UnknownParameter(name));
                }
            }
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
            arguments
        })
    }
}
#[derive(Debug, Clone)]
pub struct Group {
    /// Nom du groupe
    name: String,
    /// Description du groupe
    help: Option<String>,
    /// Role pouvant accéder le groupe. Tout le monde si None.
    permission: Option<String>,
    /// Liste des sous groupes et des commandes du groupe
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
    pub fn groups(&self) -> &Container<Group> {
        &self.node.groups
    }
    pub fn add_command(mut self, cmd: Command) -> Group {
        self.node.commands.add(cmd);
        self
    }
    pub fn commands(&self) -> &Container<Command> {
        &self.node.commands
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
                None => Err(ParseError::PartiallyNotMatched(&args[1])),
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
/// Noeud de l'arbre de commandes
/// Un groupe peut contenir des sous groupes et des commandes, stockées dans ces noeuds.
#[derive(Debug, Clone)]
pub struct Node {
    /// Liste des commandes
    pub commands: Container<Command>,
    /// Liste des sous groupes
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
/// Conteneur de commandes ou de groupes
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