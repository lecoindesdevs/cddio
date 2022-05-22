use std::{collections::HashMap, rc::Rc, cell::RefCell};
use syn::spanned::Spanned;
use super::function::{RefFunction, FunctionType};
use quote::quote;
use proc_macro2 as pm2;

use crate::util::*;

#[derive(Debug, Clone, Default)]
pub struct GroupAttribute {
    name: String,
    description: String,
    parent: Option<String>
}
impl GroupAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        let mut result = GroupAttribute::default();
        let arg_span = attr.span();
        let args = parse2::<ParenValue<MacroArgs>>(attr.tokens)?;
        for arg in args.value.args.into_iter() {
            match (arg.name.to_string().as_str(), arg.value) {
                ("name", Lit::Str(s)) => result.name = s.value(),
                ("description", Lit::Str(s)) => result.description = s.value(),
                ("parent", Lit::Str(s)) => result.parent = Some(s.value()),
                ("name"|"description"|"parent", v) => return Err(syn::Error::new_spanned(v, "String literal attendu")),
                _ => return Err(Error::new_spanned(arg.name, "Argument inconnu.")),
            }
        }
        if result.description.is_empty() {
            return Err(Error::new(arg_span, "missing description argument"));
        }
        Ok(result)
    }
}
#[derive(Debug, Clone)]
pub struct Group {
    attr: Option<GroupAttribute>,
    children: Vec<RefGroup>,
    functions: Vec<RefFunction>
}
type RefGroup = Rc<RefCell<Group>>;

impl Group {
    pub fn new_rc(attr: GroupAttribute) -> RefGroup {
        Rc::new(RefCell::new(Group {
            attr: Some(attr),
            children: Vec::new(),
            functions: Vec::new()
        }))
    }
    pub fn add_function(&mut self, function: RefFunction) {
        self.functions.push(function);
    }
    pub fn get_declarative(&self) -> pm2::TokenStream {
        let it_commands = self.functions.iter().map(|f| {
            let f_borrow = f.borrow();
            match &*f_borrow {
                FunctionType::Command(c) => c.get_declarative(),
                _ => unreachable!()
            }
        });
        let it_children = self.children.iter().map(|f| f.borrow().get_declarative());
        let node = quote! {
            opencdd_components::declarative::Node {
                commands: &[#(#it_commands), *],
                children: &[#(#it_children), *]
            }
        };

        if let Some(attr) = &self.attr {
            let name = &attr.name;
            let description = &attr.description;
            quote!(
                opencdd_components::declarative::ChildNode {
                    name: #name,
                    description: #description,
                    node: #node
                }
            )
        } else {
            node
        }
        
    }
}
impl Default for Group {
    fn default() -> Self {
        Group {
            attr: None,
            children: Vec::new(),
            functions: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupManager {
    group_map: HashMap<String, RefGroup>,
    root: Group
}

impl GroupManager {
    pub fn new() -> GroupManager {
        GroupManager {
            group_map: HashMap::new(),
            root: Group::default()
        }
    }
    pub fn from_iter<I: Iterator<Item=syn::Attribute>>(iter: I) -> syn::Result<GroupManager> {
        use syn::*;
        let mut group_manager = GroupManager::new();
        for attr in iter {
            let attr_span = attr.span();
            let group_attr = GroupAttribute::from_attr(attr)?;
            // prevent from having multiple group with the same name
            if group_manager.group_map.contains_key(&group_attr.name) {
                return Err(Error::new(attr_span, "group name already used"));
            }
            let group = Group::new_rc(group_attr.clone());
            group_manager.group_map.insert(group_attr.name, Rc::clone(&group));
            let parent = group_attr.parent;
            if let Some(parent) = &parent {
                if let Some(parent_group) = group_manager.group_map.get_mut(parent) {
                    parent_group.borrow_mut().children.push(Rc::clone(&group));
                } else {
                    return Err(Error::new(attr_span, "parent group not found"));
                }
            } else {
                group_manager.root.children.push(Rc::clone(&group));
            }
        }
        Ok(group_manager)
    }
    pub fn root(&self) -> &Group {
        &self.root
    }
    pub fn find_group(&self, name: &str) -> Option<RefGroup> {
        self.group_map.get(name).cloned()
    }
    pub fn get_declarative(&self) -> pm2::TokenStream {
        self.root.get_declarative()
    }
}