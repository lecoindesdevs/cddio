use std::{collections::HashMap, rc::Rc, cell::RefCell};
use syn::spanned::Spanned;
use super::function::Function;

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
#[derive(Debug, Clone, Default)]
pub struct Group {
    attr: GroupAttribute,
    children: Vec<RefGroup>,
    functions: Vec<Rc<Function>>
}
type RefGroup = Rc<RefCell<Group>>;

impl Group {
    pub fn new_rc(attr: GroupAttribute) -> RefGroup {
        Rc::new(RefCell::new(Group {
            attr,
            children: Vec::new(),
            functions: Vec::new()
        }))
    }
}
#[derive(Debug, Clone)]
pub struct GroupManager {
    group_map: HashMap<String, RefGroup>,
    root: RefGroup
}

impl GroupManager {
    pub fn new() -> GroupManager {
        GroupManager {
            group_map: HashMap::new(),
            root: Group::new_rc(GroupAttribute::default())
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
            }
        }
        Ok(group_manager)
    }
    pub fn root(&self) -> RefGroup {
        self.root.clone()
    }
    pub fn find_group(&self, name: &str) -> Option<RefGroup> {
        self.group_map.get(name).cloned()
    }
}