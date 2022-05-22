use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::{util::ParenValue, function::Function};
use std::fmt;
#[derive(Debug, Clone)]
pub struct EventAttribute {
    pub name: String,
}

impl EventAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        
        let arg_span = attr.span();
        let args = parse2::<ParenValue<Ident>>(attr.tokens)?;
        Ok(EventAttribute{
            name: args.value.to_string()
        })
    }
}
#[derive(Clone)]
pub struct Event {
    attr: EventAttribute,
    impl_fn: syn::ImplItemMethod,
}

impl Event {
    pub fn new(attr: syn::Attribute, impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attr = EventAttribute::from_attr(attr)?;
        Ok(Event {
            attr,
            impl_fn,
        })
    }
}
impl Function for Event {
    fn name(&self) -> proc_macro2::TokenStream {
        let name = &self.impl_fn.sig.ident;
        quote! { #name }
    }

    fn event_handle(&self) -> proc_macro2::TokenStream {
        let event_name = quote::format_ident!("{}", self.attr.name);
        let func_name = self.name();
        
        quote!{serenity::model::event::Event::#event_name(evt) => self.#func_name(ctx, evt),}
    }
}

impl ToTokens for Event {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_fn.to_tokens(tokens);
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("event", &self.attr.name)
            .finish()
    }
}