use proc_macro2 as pm2;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Reader {
    pub read_expr: pm2::TokenStream,
    pub option_type: pm2::TokenStream,
}

macro_rules! to_decl {
    ($enum_name:ident) => {
        quote!{
            serenity::model::application::command::CommandOptionType::$enum_name
        }
    };
}

impl Reader {
    pub fn argument_decode(name: &str, ty: &syn::Path) -> syn::Result<Reader> {
        use syn::*;
        
        let (ident, ty_name) = match ty.segments.last() {
            Some(segment) => (&segment.ident, segment.ident.to_string()),
            None => return Err(Error::new_spanned(ty, "Type incomplet."))
        };
        Ok(match ty_name.as_str() {
            "String" => Reader {
                read_expr: Self::reader(name, quote! {String}),
                option_type: to_decl! {String},
            },
            "str" => return Err(syn::Error::new_spanned(ty, "Utilisez String à la place.")),
            "u64" | "u32" | "u16" | "u8" 
            | "i64" | "i32" | "i16" | "i8" => Reader {
                read_expr: Self::custom_reader(name, quote! {Integer(ref s)},quote! { Some(*s as #ident) } ),
                option_type: to_decl! {Integer},
            },
            "bool" => Reader {
                read_expr: Self::reader(name, quote! {Boolean}),
                option_type: to_decl! {Boolean},
            },
            "User" => Reader {
                read_expr: Self::custom_reader(name, quote! {User(s, _)}, quote! { Some(s) }),
                option_type: to_decl! {User},
            },
            "UserId" => Reader {
                read_expr: Self::custom_reader(name, quote! {User(s, _)}, quote! { Some(s.id) }),
                option_type: to_decl! {User},
            },
            "Role" => Reader {
                read_expr: Self::reader(name, quote! {Role}),
                option_type: to_decl! {Role},
            },
            "RoleId" => Reader {
                read_expr: Self::custom_reader(name, quote! {Role(s)}, quote! { Some(s.id) }),
                option_type: to_decl! {Role},
            },
            "Mentionable" => Reader {
                read_expr: Self::mentionable_reader(name),
                option_type: to_decl! {Mentionable},
            },
            "PartialChannel" => Reader{
                read_expr: Self::reader(name, quote! {Channel}),
                option_type: to_decl! {Channel},
            },
            "ChannelId" => Reader {
                read_expr: Self::custom_reader(name, quote! {Channel(s)}, quote! { Some(s.id) }),
                option_type: to_decl! {Channel},
            },
            "f64" | "f32" => Reader {
                read_expr: Self::custom_reader(name, quote! {Number(s)}, quote! { Some(*s as #ty) } ),
                option_type: to_decl! {Number},
            } ,
            _ => return Err(Error::new_spanned(ty, "Type d'argument incompatible.")),
        })
    }
    fn new(expr: pm2::TokenStream,declarative: pm2::TokenStream) -> Reader {
        Reader { read_expr: expr, option_type: declarative }
    }
    fn custom_reader(name: &str, ty: pm2::TokenStream, expr: pm2::TokenStream) -> pm2::TokenStream {
        quote! {
            match app_command.get_argument(#name) {
                Some(serenity::model::application::interaction::application_command::CommandDataOption{
                    resolved: Some(serenity::model::application::interaction::application_command::CommandDataOptionValue::#ty),
                    ..
                }) => {#expr},
                _ => None
            }
        }
    }
    fn mentionable_reader(name: &str) -> pm2::TokenStream {
        quote! {
            match app_command.get_argument(#name) {
                Some(serenity::model::application::interaction::application_command::CommandDataOption{
                    resolved: Some(serenity::model::application::interaction::application_command::CommandDataOptionValue::User(s, _)),
                    ..
                }) => {Some(cddio_core::embed::Mentionable::User(s.id))},
                Some(serenity::model::application::interaction::application_command::CommandDataOption{
                    resolved: Some(serenity::model::application::interaction::application_command::CommandDataOptionValue::Role(s)),
                    ..
                }) => {Some(cddio_core::embed::Mentionable::Role(s.id))},
                _ => None
            }
        }
    }
    fn reader(name: &str, ty: pm2::TokenStream) -> pm2::TokenStream {
        Self::custom_reader(name, quote!{#ty (s)}, quote! { Some(s) })
    }
}