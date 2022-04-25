use quote::{quote, ToTokens};
use proc_macro2 as pm2;
use super::util::*;

pub enum DeclarativeType {
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
    Attachment,
}

#[derive(Debug, Clone)]
pub enum ArgumentType {
    Parameter{
        call_variable: pm2::TokenStream,
        decode_expr: pm2::TokenStream,
        description: String,
        optional: bool,
    },
    Internal{
        call_variable: pm2::TokenStream,
    },
    SelfArg,
}
#[derive(Debug, Clone)]
pub struct Argument { 
    base: syn::FnArg,
    arg_type: ArgumentType,
}

impl Argument {
    pub fn new(arg: syn::FnArg) -> syn::Result<Argument> {
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
                                optional: true,
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
    pub fn argument_decode(name: &str, ty: &syn::Path) -> syn::Result<pm2::TokenStream> {
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
            "User" => Self::make_argument_custom_getter(name, quote! {User(s, _)}, quote! { Some(s.0) }),
            "UserId" => Self::make_argument_custom_getter(name, quote! {User(s, _)}, quote! { Some(s.id) }),
            "Role" => Self::make_argument_getter(name, quote! {Role}),
            "RoleId" => Self::make_argument_custom_getter(name, quote! {Role(s)}, quote! { Some(s.id) }),
            "Mentionable" => Self::make_argument_mentionable(name),
            "PartialChannel" => Self::make_argument_getter(name, quote! {Channel}),
            "ChannelId" => Self::make_argument_custom_getter(name, quote! {Channel(s)}, quote! { Some(s.id) }),
            "f64" | "f32" => Self::make_argument_custom_getter(name, quote! {Float(s)}, quote! { Some(s as #ty) } ),
            _ => return Err(Error::new_spanned(ty, "Type d'argument incompatible.")),
        })
    }
    fn make_argument_custom_getter(name: &str, ty: pm2::TokenStream, expr: pm2::TokenStream) -> pm2::TokenStream {
        quote! {
            match app_command.get_argument(#name) {
                Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                    resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::#ty),
                    ..
                }) => {#expr},
                _ => None
            }
        }
    }
    fn make_argument_mentionable(name: &str) -> pm2::TokenStream {
        quote! {
            match app_command.get_argument(#name) {
                Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                    resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::User(s, _)),
                    ..
                }) => {Mentionable::User(s.id)},
                Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption{
                    resolved: Some(serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::Role(s)),
                    ..
                }) => {Mentionable::Role(s.id)},
                _ => None
            }
        }
    }
    fn make_argument_getter(name: &str, ty: pm2::TokenStream) -> pm2::TokenStream {
        Self::make_argument_custom_getter(name, quote!{#ty (s)}, quote! { Some(s) })
    }
    }
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let base = &self.base;
        tokens.extend(quote! {#base});
    }
}