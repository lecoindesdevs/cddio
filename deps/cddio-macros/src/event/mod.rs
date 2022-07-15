use quote::{quote, ToTokens};

use crate::util;
use crate::{util::ParenValue, function::Function};
use std::fmt;
#[derive(Debug, Clone)]
pub struct EventAttribute {
    pub pattern: syn::Pat,
}

#[derive(Debug, Clone)]
pub enum EventPattern {
    Ident(syn::Ident),
    Pattern(syn::Pat),
}

impl syn::parse::Parse for EventPattern  {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(value) = input.parse::<syn::Ident>() {
            Ok(EventPattern::Ident(value))
        } else if let Ok(value) = input.parse::<syn::Pat>() {
            Ok(EventPattern::Pattern(value))
        } else {
            Err(input.error("Expected identifier or pattern"))
        }
    }
}

impl EventAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        let args = parse2::<ParenValue<Pat>>(attr.tokens)?;
        Ok(EventAttribute{
            pattern: args.value
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

    fn event_handle(&self) -> syn::Result<proc_macro2::TokenStream> {
        let func_name = self.name();
        
        match &self.attr.pattern {
            syn::Pat::Ident(ident) => Ok(quote! {
                serenity::model::event::Event::#ident(evt) => self.#func_name(ctx, evt).await
            }),
            pat => {
                let args = util::fn_args_to_args_call(&self.impl_fn.sig.inputs)?;
                Ok(quote! {#pat => self.#func_name(#args).await})
            }
        }
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
            .field("event", &self.attr.pattern)
            .finish()
    }
}