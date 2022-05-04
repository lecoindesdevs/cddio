use std::fmt;
use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use syn::spanned::Spanned;
use super::util::*;
use super::argument::{Argument, ArgumentType};

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

#[derive(Debug, Clone)]
pub enum FunctionType {
    Command(CommandAttribute),
    Event,
    NoSpecial,
}

#[derive(Clone)]
pub struct Function {
    pub impl_fn: syn::ImplItemMethod,
    pub fn_type: FunctionType,
    pub args: Vec<Argument>,
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
            (Some(attr), None) => FunctionType::Command(CommandAttribute::from_attr(attr)?),
            (None, Some(_)) => FunctionType::Event,
            (None, None) => FunctionType::NoSpecial,
        };
        impl_fn.attrs = attrs;
        let args = impl_fn.sig.inputs.iter().cloned().map(|arg| Argument::new(arg)).collect::<Result<Vec<_>, _>>()?;
        Ok(Function { 
            impl_fn, 
            fn_type: type_,
            args,
        })
    }
    pub fn function_name(&self) -> &syn::Ident {
        &self.impl_fn.sig.ident
    }
    pub fn command_name(&self) -> String {
        match self.fn_type {
            FunctionType::Command(ref attr) => attr.name.clone().unwrap_or_else(|| self.function_name().to_string()),
            _ => self.function_name().to_string(),
        }
    }
    pub fn function_call_event(&self) -> syn::Result<pm2::TokenStream> {
        let name = &self.function_name();
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
                args: &[
                    #(#arguments),*
                ],
            }
        }
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

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.fn_type {
            FunctionType::Command(ref attr) => {
                f.debug_struct("Command")
                    .field("name", &attr.name)
                    .field("description", &attr.description)
                    .field("function_name", &self.function_name())
                    .field("args", &self.args)
                    .finish()
            },
            FunctionType::Event => f.debug_struct("Event").finish_non_exhaustive(),
            FunctionType::NoSpecial => f.debug_struct("NoSpecial").finish(),
        }
    }
}