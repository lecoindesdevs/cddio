use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use super::util::*;

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

#[derive(Debug, Clone)]
enum ArgumentType {
    Parameter{
        call_variable: pm2::TokenStream,
        decode_expr: pm2::TokenStream,
        description: String,
    },
    Internal{
        call_variable: pm2::TokenStream,
    },
    SelfArg,
}
#[derive(Debug, Clone)]
struct Argument { 
    base: syn::FnArg,
    arg_type: ArgumentType,
}

impl Argument {
    fn new(arg: syn::FnArg) -> syn::Result<Argument> {
        use syn::*;
        match arg {
            syn::FnArg::Typed(arg) => {
                let arg_name = match arg.pat.as_ref() {
                    Pat::Ident(ident) => ident.ident.clone(),
                    _ => return Err(syn::Error::new_spanned(arg.pat, "Argument de fonction attendu."))
                };
                let arg_name_str = arg_name.to_string();
                let ty = match arg.ty.as_ref() {
                    Type::Path(TypePath { path, .. }) => path,
                    Type::Reference(TypeReference { elem, .. }) => match elem.as_ref() {
                        Type::Path(TypePath { path, .. }) => path,
                        _ => return Err(syn::Error::new_spanned(arg.ty, "Type d'argument innatendu."))
                    },
                    _ => return Err(syn::Error::new_spanned(arg.ty, "Type d'argument innatendu."))
                }.clone();
                let ty_last = match ty.segments.last() {
                    Some(segment) => segment,
                    None => return Err(syn::Error::new_spanned(ty, "discord_argument: Erreur innatendu."))
                };
                let ty_name = ty_last.ident.to_string();
                let (attr_desc, attrs): (_, Vec<_>) = arg.attrs.find_and_pop(|attr| attr.path.is_ident("description"));
                
                // let attrs = attrs.into_iter().map(|attr| quote!{#attr}).collect();
                let arg = syn::FnArg::Typed(syn::PatType { 
                    attrs,
                    ..arg
                });
                match ty_name.to_string().as_str() {
                    "Option" => {
                        let inner_ty = match &ty_last.arguments {
                            PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
                                match args.args.first().unwrap() {
                                    GenericArgument::Type(Type::Path(ref p)) => &p.path,
                                    _ => return Err(syn::Error::new_spanned(&args.args, "Type chemin attendu."))
                                }
                            },
                            _ => return Err(syn::Error::new_spanned(ty, "Mauvaise déclaration de Option. Utilisation: Option<Type>"))
                        };
                        let value_decoded = Self::argument_decode(&arg_name_str, &inner_ty)?;
                        Ok(Argument {
                            arg_type: ArgumentType::Parameter{
                                call_variable: quote!{#arg_name},
                                decode_expr: quote! { let #arg_name =  #value_decoded.cloned(); },
                                description: Self::get_description(attr_desc).or_else(|_| Err(syn::Error::new_spanned(&arg, "attribut description manquant. Utilisation: description(\"...\").")))?,
                            },
                            base: arg,
                        })
                    }
                    "ApplicationCommandEmbed" => {
                        Ok(Argument {
                            base: arg,
                            arg_type: ArgumentType::Internal{
                                call_variable: quote!{&app_command},
                            },
                        })
                    }
                    "Context" => {
                        Ok(Argument {
                            base: arg,
                            arg_type: ArgumentType::Internal{
                                call_variable: quote!{&ctx},
                            },
                        })
                    }
                    _ => {
                        let value_decoded = Self::argument_decode(&arg_name_str, &ty)?;
                        let error_msg = format!("Argument \"{}\" manquant.", arg_name_str);
                        Ok(Argument {
                            arg_type: ArgumentType::Parameter{
                                call_variable: quote!{#arg_name},
                                decode_expr: quote! { let #arg_name =  #value_decoded.ok_or_else(|| #error_msg).unwrap().to_owned(); },
                                description: Self::get_description(attr_desc).or_else(|_| Err(syn::Error::new_spanned(&arg, "attribut description manquant. Utilisation: description(\"...\").")))?,
                            },
                            base: arg,
                        })
                    }
                }
            },
            syn::FnArg::Receiver(v) => Ok(Argument {
                base: syn::FnArg::Receiver(v),
                arg_type: ArgumentType::SelfArg,
            })
        }
    }
    fn get_description(attr: Option<syn::Attribute>) -> Result<String, Option<syn::Error>> {
        match attr {
            Some(syn::Attribute { tokens, .. }) => {
                match syn::parse2::<ParenValue<syn::LitStr>>(tokens.clone()) {
                    Ok(item) => Ok(item.value.value()),
                    Err(_) => Err(Some(syn::Error::new_spanned(tokens, "attribut description mal formé. Utilisation: description(\"...\").")))
                }
            },
            None => Err(None),
        }
    }
    fn make_argument_custom_getter(name: &str, ty: pm2::TokenStream, expr: pm2::TokenStream) -> pm2::TokenStream {
        quote! {
            match app_command.get_argument(#name) {
                Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                    resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::#ty(s)),
                    ..
                }) => {#expr},
                _ => None
            }
        }
    }
    fn make_argument_getter(name: &str, ty: pm2::TokenStream) -> pm2::TokenStream {
        Self::make_argument_custom_getter(name, ty, quote! { Some(s) })
    }
    fn argument_decode(name: &str, ty: &syn::Path) -> syn::Result<pm2::TokenStream> {
        use syn::*;
        let ty_name = match ty.get_ident() {
            Some(ident) => ident.to_string(),
            None => return Err(Error::new_spanned(ty, "Type incomplet."))
        };
        Ok(match ty_name.as_str() {
            "String" => Self::make_argument_getter(name, quote! {String}),
            "str" => return Err(syn::Error::new_spanned(ty, "Utilisez String à la place.")),
            "u64" | "u32" | "u16" | "u8" 
            | "i64" | "i32" | "i16" | "i8" => Self::make_argument_custom_getter(name, quote! {Integer},quote! { Some(s as #ty) } ),
            "bool" => Self::make_argument_getter(name, quote! {Boolean}),
            "UserId" => Self::make_argument_custom_getter(name, quote! {User}, quote! { Some(s.id) }),
            "ChannelId" => Self::make_argument_custom_getter(name, quote! {Channel}, quote! { Some(s.id) }),
            "RoleId" => Self::make_argument_custom_getter(name, quote! {Role}, quote! { Some(s.id) }),
            "f64" | "f32" => Self::make_argument_custom_getter(name, quote! {Float}, quote! { Some(s as #ty) } ),
            _ => return Err(Error::new_spanned(ty, "Type d'argument incompatible.")),
        })
    }
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let base = &self.base;
        tokens.extend(quote! {#base});
    }
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
        let sig = &self.impl_fn.sig;
        let mut args_call = vec![];
        let mut args_decode = vec![];
        for arg in self.args.iter() {
            match &arg.arg_type {
                ArgumentType::Parameter{call_variable, decode_expr, ..} => {
                    args_decode.push(decode_expr);
                    args_call.push(call_variable)
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