use quote::ToTokens;
use syn::spanned::Spanned;
use std::fmt;
use quote::quote;
use crate::util::{self, MacroArgs};

use crate::{util::ParenValue, function::Function};

#[derive(Debug, Clone, Default)]
pub struct InteractionAttribute {
    custom_id: String,
}
pub struct Interaction {
    attr: InteractionAttribute,
    impl_fn: syn::ImplItemMethod,
}

impl InteractionAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        let attr_span = attr.span();
        let mut result = Self::default();
        let args = parse2::<ParenValue<MacroArgs>>(attr.tokens)?;
        for arg in args.value.args.into_iter() {
            match (arg.name.to_string().as_str(), arg.value) {
                ("custom_id", Lit::Str(s)) => result.custom_id = s.value(),
                _ => return Err(Error::new_spanned(arg.name, "Argument inconnu ou mal typé.")),
            }
        }
        if result.custom_id.is_empty() {
            return Err(Error::new(attr_span, "Argument custom_id manquant"));
        }
        Ok(result)
    }
}

impl Interaction {
    pub fn new(attr: syn::Attribute, impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attr = InteractionAttribute::from_attr(attr)?;
        Ok(Interaction {
            attr,
            impl_fn,
        })
    }
}

impl Function for Interaction {
    fn name(&self) -> proc_macro2::TokenStream {
        let name = &self.impl_fn.sig.ident;
        quote! { #name }
    }

    fn event_handle(&self) -> syn::Result<proc_macro2::TokenStream> {
        let func_name = self.name();
        let custom_id = &self.attr.custom_id;
        let fn_args = util::fn_args_to_args_call(&self.impl_fn.sig.inputs)?;
        Ok(quote!{
            serenity::model::event::Event::InteractionCreate(serenity::model::event::InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::MessageComponent(message_interaction), ..}) if message_interaction.data.custom_id == #custom_id => self.#func_name(#fn_args).await
        })
    }
}

impl ToTokens for Interaction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.impl_fn.to_tokens(tokens);
    }
}

impl fmt::Debug for Interaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MsgComponent")
            .field("custom_id", &self.attr.custom_id)
            .finish()
    }
}