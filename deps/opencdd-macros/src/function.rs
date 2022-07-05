use std::cell::RefCell;
use std::rc::Rc;

use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use super::util::*;
use super::command::Command;
use super::event::Event;
use super::message_component::Interaction;

pub trait Function : ToTokens + std::fmt::Debug {
    fn name(&self) -> pm2::TokenStream;
    fn event_handle(&self) -> syn::Result<pm2::TokenStream>;
}

#[derive(Debug, Clone)]
pub struct NoSpecial(syn::ImplItemMethod);

impl Function for NoSpecial {
    fn name(&self) -> pm2::TokenStream {
        self.0.sig.ident.to_token_stream()
    }
    fn event_handle(&self) -> syn::Result<pm2::TokenStream> {
        Ok(quote! {})
    }
}
impl ToTokens for NoSpecial {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}
#[derive(Debug)]
pub enum FunctionType {
    Command(Command),
    Event(Box<dyn Function>),
    NoSpecial(NoSpecial),
}

pub type RefFunction = Rc<RefCell<FunctionType>>;

impl Function for FunctionType {
    fn name(&self) -> pm2::TokenStream {
        match self {
            FunctionType::Command(c) => c.name(),
            FunctionType::Event(e) => e.name(),
            FunctionType::NoSpecial(n) => n.name(),
        }
    }
    fn event_handle(&self) -> syn::Result<pm2::TokenStream> {
        match self {
            FunctionType::Command(c) => c.event_handle(),
            FunctionType::Event(e) => e.event_handle(),
            FunctionType::NoSpecial(n) => n.event_handle(),
        }
    }
}
impl ToTokens for FunctionType {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        match self {
            FunctionType::Command(c) => c.to_tokens(tokens),
            FunctionType::Event(e) => e.to_tokens(tokens),
            FunctionType::NoSpecial(n) => n.to_tokens(tokens),
        }
    }
}
impl FunctionType {

    pub fn new(mut impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attrs = impl_fn.attrs.clone();
        const LIST_ATTR: [&str; 3] = ["command", "event", "message_component"];
        
        if attrs.iter().filter(|a| LIST_ATTR.contains(&a.path.to_token_stream().to_string().as_str())).count() > 1 {
            return Err(syn::Error::new_spanned(impl_fn.sig.ident, "Only one discord event type attribute is allowed"));
        }
        let finder = |name: &'static str| {
            move |attr: &syn::Attribute| {
                match attr.path.get_ident() {
                    Some(ident) => ident.to_string() == name,
                    None => return false
                }
            }
        };
        // Check if the function is a command
        let (attr_cmd, attrs): (_, Vec<_>) = attrs.find_and_pop(finder("command"));
        if let Some(attr_cmd) = attr_cmd {
            impl_fn.attrs = attrs;
            let cmd = Command::new(attr_cmd, impl_fn)?;
            return Ok(FunctionType::Command(cmd));
        }
        // Check if the function is an event
        let (attr_evt, attrs): (_, Vec<_>) = attrs.find_and_pop(finder("event"));
        if let Some(attr_evt) = attr_evt {
            impl_fn.attrs = attrs;
            let evt = Event::new(attr_evt, impl_fn)?;
            return Ok(FunctionType::Event(Box::new(evt)));
        }
        // Check if the function is an message_component event
        let (attr_evt, attrs): (_, Vec<_>) = attrs.find_and_pop(finder("message_component"));
        if let Some(attr_evt) = attr_evt {
            impl_fn.attrs = attrs;
            let evt = Interaction::new(attr_evt, impl_fn)?;
            return Ok(FunctionType::Event(Box::new(evt)));
        }
        // Otherwise, it's a no special function
        Ok(FunctionType::NoSpecial(NoSpecial(impl_fn)))
    }
    pub fn new_rc(mut impl_fn: syn::ImplItemMethod) -> syn::Result<RefFunction> {
       Self::new(impl_fn).map(|f| Rc::new(RefCell::new(f)))
    }
}