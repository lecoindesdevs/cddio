mod argument;
use crate::util::{ParenValue, MacroArgs};
use proc_macro2 as pm2;
use std::fmt;
use quote::{ToTokens, quote};
use self::argument::ArgumentType;

use super::Function;


#[derive(Debug, Clone, Default)]
pub struct CommandAttribute {
    pub name: Option<String>,
    pub description: String,
    pub group: Option<String>
}
impl CommandAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        let mut result = CommandAttribute::default();
        let arg_span = attr.span();
        let args = parse2::<ParenValue<MacroArgs>>(attr.tokens)?;
        for arg in args.value.args.into_iter() {
            match (arg.name.to_string().as_str(), arg.value) {
                ("name", Lit::Str(s)) => result.name = Some(s.value()),
                ("description", Lit::Str(s)) => result.description = s.value(),
                ("group", Lit::Str(s)) => result.group = Some(s.value()),
                ("name"|"description"|"group", v) => return Err(syn::Error::new_spanned(v, "String literal attendu")),
                _ => return Err(Error::new_spanned(arg.name, "Argument inconnu.")),
            }
        }
        if result.description.is_empty() {
            return Err(Error::new(arg_span, "missing description argument"));
        }
        Ok(result)
    }
}

pub struct Command {
    pub attr: CommandAttribute,
    pub impl_fn: syn::ImplItemMethod,
    pub args: Vec<argument::Argument>,
}

impl Function for Command {
    fn name(&self) -> pm2::TokenStream {
        let name = &self.impl_fn.sig.ident;
        quote! { #name }
    }

    fn event_handle(&self) -> pm2::TokenStream {
        let name = self.name();
        let mut args_call = vec![];
        let mut args_decode = vec![];
        for arg in self.args.iter() {
            match &arg.get_type() {
                ArgumentType::Parameter{call_variable, reader, ..} => {
                    args_decode.push(&reader.read_expr);
                    args_call.push(call_variable);
                },
                ArgumentType::Internal { call_variable } => args_call.push(call_variable),
                ArgumentType::SelfArg => continue,
            }
        }
        quote! {
            #(#args_decode)*
            self.#name(#(#args_call),*)
        }
    }
}

impl ToTokens for Command {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let syn::Signature { 
            abi,
            unsafety,
            constness,
            asyncness,
            ident,
            // inputs,
            output,
            ..
        } = &self.impl_fn.sig;
        let body = &self.impl_fn.block;
        let inputs = &self.args;
        let attrs = &self.impl_fn.attrs;
        tokens.extend(quote! { #(#attrs)* #unsafety #constness #asyncness #abi fn #ident (#(#inputs), *) #output #body });
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.attr.name)
            .field("description", &self.attr.description)
            .field("function_name", &self.function_name())
            .field("args", &self.args)
            .finish()
    }
}