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

macro_rules! to_event {
    ($impl_fn:ident, $(($title:expr, $event_type:ty, $function_type:ident)), *) => {
        {
            let mut attrs = $impl_fn.attrs.clone();
            let func = 'a: loop{
                $(
                    let find = |attr: &syn::Attribute| {
                        match attr.path.get_ident() {
                            Some(ident) => ident.to_string() == $title,
                            None => return false
                        }
                    };
                    let (attr_evt, other): (_, Vec<_>) = attrs.find_and_pop(find);
                    if let Some(attr_evt) = attr_evt {
                        $impl_fn.attrs = other;
                        let evt = <$event_type>::new(attr_evt, $impl_fn)?;
                        break 'a FunctionType::$function_type(to_event!(evt, _result => $function_type));
                    } else {
                        attrs = other;
                    }
                )*
                break 'a FunctionType::NoSpecial(NoSpecial($impl_fn));
            };
            Ok(func)
        }
    };
    ($evt:ident, _result => Command) => {
        $evt
    };
    ($evt:ident, _result => Event) => {
        Box::new($evt)
    };
}

impl FunctionType {
    #[allow(unused_assignments)]
    pub fn new(mut impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        to_event!(impl_fn, 
            ("command", Command, Command),
            ("event", Event, Event),
            ("message_component", Interaction, Event)
        )
    }
    pub fn new_rc(impl_fn: syn::ImplItemMethod) -> syn::Result<RefFunction> {
       Self::new(impl_fn).map(|f| Rc::new(RefCell::new(f)))
    }
}