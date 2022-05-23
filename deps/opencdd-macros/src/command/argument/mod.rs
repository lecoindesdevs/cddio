mod reader;
mod attribute;
use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use syn::spanned::Spanned;
use crate::util::*;
pub use reader::*;
pub use attribute::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ArgumentType {
    Parameter{
        call_variable: pm2::TokenStream,
        reader: Reader,
        attribute: ArgumentAttribute,
        optional: bool,
    },
    Internal{
        call_variable: pm2::TokenStream,
    },
    SelfArg,
}
#[derive(Clone)]
pub struct Argument { 
    base: syn::FnArg,
    arg_type: ArgumentType,
}

impl Argument {
    pub fn new(arg: syn::FnArg) -> syn::Result<Argument> {
        use syn::*;
        let arg_span = arg.span();
        match arg {
            syn::FnArg::Typed(arg) => {
                let var_name = match arg.pat.as_ref() {
                    Pat::Ident(ident) => ident.ident.clone(),
                    _ => return Err(syn::Error::new_spanned(arg.pat, "Argument de fonction attendu."))
                };
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
                let (attr_desc, attrs): (_, Vec<_>) = arg.attrs.find_and_pop(|attr| attr.path.is_ident("argument"));
                
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
                        let attribute = match attr_desc {
                            Some(attr) => ArgumentAttribute::from_attr(attr)?,
                            None => return Err(syn::Error::new(arg_span, "discord_argument: Attribut 'argument' manquant."))
                        };
                        let arg_name = match attribute.name.clone() {
                            Some(name) => name,
                            None => var_name.to_string()
                        };
                        let call_variable = &var_name;
                        let value_decoded = Reader::argument_decode(&arg_name, &inner_ty)?;
                        Ok(Argument {
                            arg_type: ArgumentType::Parameter{
                                call_variable: quote!{#call_variable},
                                reader: {
                                    let expr = value_decoded.read_expr;
                                    Reader{
                                        read_expr: quote! { let #call_variable =  #expr.cloned(); },
                                        .. value_decoded
                                    }
                                },
                                attribute,
                                optional: true,
                            },
                            base: arg,
                        })
                    }
                    "ApplicationCommandEmbed" => {
                        Ok(Argument {
                            base: arg,
                            arg_type: ArgumentType::Internal{
                                call_variable: quote!{app_command},
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
                        let attribute = match attr_desc {
                            Some(attr) => ArgumentAttribute::from_attr(attr)?,
                            None => return Err(syn::Error::new(arg_span, "discord_argument: Attribut 'argument' manquant."))
                        };
                        let arg_name = match attribute.name.clone() {
                            Some(name) => name,
                            None => var_name.to_string()
                        };
                        let value_decoded = Reader::argument_decode(&arg_name, &ty)?;
                        let error_msg = format!("Argument \"{}\" manquant.", arg_name);
                        Ok(Argument {
                            arg_type: ArgumentType::Parameter{
                                call_variable: quote!{#var_name},
                                reader: {
                                    let expr = value_decoded.read_expr;
                                    Reader{
                                        read_expr: quote! { let #var_name =  #expr.ok_or_else(|| #error_msg).unwrap().to_owned(); },
                                        .. value_decoded
                                    }
                                },
                                attribute,
                                optional: false,
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
    pub fn get_description(attr: Option<syn::Attribute>) -> Result<String, Option<syn::Error>> {
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
    pub fn get_type(&self) -> &ArgumentType {
        &self.arg_type
    }
    pub fn get_declarative(&self) -> Option<pm2::TokenStream> {
        let (attr, optional, option_type) = match &self.arg_type {
            ArgumentType::Parameter{attribute, optional, reader, ..} => (attribute, optional, &reader.option_type),
            _ => return None
        };
        let name = attr.name.clone();
        let name = match (name, &self.base) {
            (Some(name), _) => name,
            (None, syn::FnArg::Typed(syn::PatType{ref pat, ..})) => {
                let name = match &pat.as_ref() {
                    syn::Pat::Ident(syn::PatIdent{ref ident, ..}) => ident.to_string(),
                    _ => return None
                };
                name
            },
            _ => return None
        };
        let description = &attr.description;
        Some(quote! {
            opencdd_components::declarative::Argument{
                name: #name,
                type_: #option_type,
                description: #description,
                optional: #optional,
            }
        })
    }
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let base = &self.base;
        tokens.extend(quote! {#base});
    }
}
impl fmt::Debug for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.arg_type {
            ArgumentType::Parameter{ optional, attribute, call_variable, ..} => {
                let mut f_struct = f.debug_struct("Argument");
                if let Some(name) = &attribute.name {
                    f_struct.field("name", name);
                } else {
                    f_struct.field("name", &call_variable.to_string());
                }
                    // .field("type", &self.)
                f_struct.field("description", &attribute.description)
                    .field("optional", optional)
                    .finish()
            },
            ArgumentType::Internal{call_variable} => {
                f.debug_tuple("InternalParameter")
                    .field(&call_variable.to_string())
                    .finish()
            },
            ArgumentType::SelfArg => f.debug_tuple("SelfParameter").finish(),
        }
    }
}