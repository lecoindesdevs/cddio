use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use super::util::*;
use super::argument::{Argument, ArgumentType};

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Command,
    Event,
    NoSpecial,
}

#[derive(Debug, Clone)]
pub struct Function {
    impl_fn: syn::ImplItemMethod,
    type_: FunctionType,
    args: Vec<Argument>,
}


impl Function {
    pub fn new(mut impl_fn: syn::ImplItemMethod) -> syn::Result<Function> {
        let attrs = impl_fn.attrs.clone();
        let (attr_cmd, attrs): (_, Vec<_>) = attrs.find_and_pop(|attr| {
            match attr.path.get_ident() {
                Some(ident) => ident.to_string() == "command",
                None => return false
            }
        });
        let (attr_evt, attrs): (_, Vec<_>) = attrs.find_and_pop(|attr| {
            match attr.path.get_ident() {
                Some(ident) => ident.to_string() == "event",
                None => return false
            }
        });
        let type_ = match (attr_cmd, attr_evt) {
            (Some(_), Some(_)) => return Err(syn::Error::new_spanned(&impl_fn, "Commande et événement ne peuvent pas être déclarés en même temps.")),
            (Some(_), None) => FunctionType::Command,
            (None, Some(_)) => FunctionType::Event,
            (None, None) => FunctionType::NoSpecial,
        };
        impl_fn.attrs = attrs;
        let args = impl_fn.sig.inputs.iter().cloned().map(|arg| Argument::new(arg)).collect::<Result<Vec<_>, _>>()?;
        Ok(Function { 
            impl_fn, 
            type_,
            args,
        })
    }
    pub fn function_name(&self) -> &syn::Ident {
        &self.impl_fn.sig.ident
    }
    pub fn function_call_event(&self) -> syn::Result<pm2::TokenStream> {
        let name = &self.function_name();
        let mut args_call = vec![];
        let mut args_decode = vec![];
        for arg in self.args.iter() {
            match &arg.get_type() {
                ArgumentType::Parameter{call_variable, decoded, ..} => {
                    args_decode.push(&decoded.expr);
                    args_call.push(call_variable);
                },
                ArgumentType::Internal { call_variable } => args_call.push(call_variable),
                ArgumentType::SelfArg => continue,
            }
        }
        Ok(quote! {
            #(#args_decode)*
            self.#name(#(#args_call),*)
        })
    }
    pub fn get_declarative(&self) -> pm2::TokenStream {
        let arguments = self.args.iter().filter_map(|v| v.get_declarative());
        let name = self.function_name().to_string();
        quote! {
            Command {
                name: #name,
                description: "",
                params: &[
                    #(#arguments),*
                ],
            }
        }
    }
    pub fn is(&self, ftype: FunctionType) -> bool {
        self.type_ == ftype
    }
}

impl ToTokens for Function {
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