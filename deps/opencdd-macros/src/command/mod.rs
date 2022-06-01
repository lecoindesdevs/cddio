mod argument;
use crate::util::{ParenValue, MacroArgs};
use proc_macro2 as pm2;
use syn::spanned::Spanned;
use std::fmt;
use quote::{ToTokens, quote};
use self::argument::{ArgumentType, Argument};

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

#[derive(Clone)]
pub struct Command {
    pub attr: CommandAttribute,
    pub impl_fn: syn::ImplItemMethod,
    pub args: Vec<argument::Argument>,
}

impl Command {
    pub fn new(attr: syn::Attribute, impl_fn: syn::ImplItemMethod) -> syn::Result<Self> {
        let attr = CommandAttribute::from_attr(attr)?;
        let args = impl_fn.sig.inputs.iter().cloned().map(|arg| Argument::new(arg)).collect::<Result<Vec<_>, _>>()?;
        Ok(Command {
            attr,
            impl_fn,
            args,
        })
    }
    pub fn get_declarative(&self) -> Option<pm2::TokenStream> {
        let arguments = self.args.iter().filter_map(|v| v.get_declarative());
        let name = match self.attr.name {
            Some(ref name) => name.clone(),
            None => self.name().to_string(),
        }; 
        let description = &self.attr.description;
        Some(
            quote! {
                opencdd_components::declarative::Command {
                    name: #name,
                    description: #description,
                    args: &[
                        #(#arguments),*
                    ],
                }
            }
        )
    }
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
            self.#name(#(#args_call),*).await;
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
        let syn::ImplItemMethod {
            attrs,
            vis,
            defaultness,
            block,
            ..
        } = &self.impl_fn;
        let inputs = &self.args;
        tokens.extend(quote! { #(#attrs)* #vis #defaultness #unsafety #constness #asyncness #abi fn #ident (#(#inputs), *) #output #block });
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.attr.name)
            .field("description", &self.attr.description)
            .field("function_name", &self.name())
            .field("args", &self.args)
            .finish()
    }
}