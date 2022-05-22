use std::cell::RefCell;
use std::rc::Rc;

use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use super::util::*;
use super::command::Command;
use super::event::Event;

pub trait Function : ToTokens {
    fn name(&self) -> pm2::TokenStream;
    fn event_handle(&self) -> pm2::TokenStream;
}

type RefFunction = Rc<RefCell<dyn Function>>;

struct NoSpecial(syn::ImplItemMethod);

impl Function for NoSpecial {
    fn name(&self) -> pm2::TokenStream {
        self.0.sig.ident.to_token_stream()
    }
    fn event_handle(&self) -> pm2::TokenStream {
        quote! {}
    }
}
impl ToTokens for NoSpecial {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

fn make_function(mut impl_fn: syn::ImplItemMethod) -> syn::Result<RefFunction> {
    
    let attrs = impl_fn.attrs.clone();
    const LIST_ATTR: [&str; 2] = ["command", "event"];
    
    if attrs.iter().filter(|a| LIST_ATTR.contains(&a.path.to_token_stream().to_string().as_str())).count() > 1 {
        return Err(syn::Error::new_spanned(impl_fn.sig.ident, "Only one discord event type attribute is allowed"));
    }
    let finder = |name: &str| {
        |attr: &syn::Attribute| {
            match attr.path.get_ident() {
                Some(ident) => ident.to_string() == name,
                None => return false
            }
        }
    };
    let (attr_cmd, attrs): (_, Vec<_>) = attrs.find_and_pop(finder("command"));
    if let Some(attr_cmd) = attr_cmd {
        impl_fn.attrs = attrs;
        let cmd = Command::new(attr_cmd, impl_fn)?;
        return Ok(Rc::new(RefCell::new(cmd)));
    }
    let (attr_evt, attrs): (_, Vec<_>) = attrs.find_and_pop(finder("event"));
    if let Some(attr_evt) = attr_evt {
        impl_fn.attrs = attrs;
        let evt = Event::new(attr_evt, impl_fn)?;
        return Ok(Rc::new(RefCell::new(evt)));
    }
    Ok(Rc::new(RefCell::new(NoSpecial(impl_fn))))
}