use quote::quote;
use proc_macro2 as pm2;
#[derive(Debug, Clone)]
pub struct Function {
    pub attributes: Vec<syn::Attribute>,
    pub signature: syn::Signature,
    pub body: syn::Block,
}
struct Argument { 
    variable_name: pm2::TokenStream,
    decode_expr: Option<pm2::TokenStream>,
    is_self: bool,
}

impl Function {
    pub fn function_name(&self) -> &syn::Ident {
        &self.signature.ident
    }
    pub fn function_call_event(&self) -> syn::Result<pm2::TokenStream> {
        let name = &self.signature.ident;
        let args = self.signature.inputs.iter();
        let mut args_call = vec![];
        let mut args_decode = vec![];
        for arg in args {
            let Argument { variable_name, decode_expr, is_self } = Self::discord_argument(arg)?;
            if !is_self {
                args_call.push(quote! { #variable_name });
            }
            if let Some(decode_expr) = decode_expr {
                args_decode.push(quote! { #decode_expr });
            }
        }
        Ok(quote! {
            #(#args_decode)*
            self.#name(#(#args_call),*)
        })
    }
    pub fn function_decl(&self) -> pm2::TokenStream {
        let signature = &self.signature;
        let body = &self.body;
        quote! { #signature #body }
    }
    fn discord_argument(arg: &syn::FnArg) -> syn::Result<Argument> {
        use syn::*;
        match arg {
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                let variable = match pat.as_ref() {
                    Pat::Ident(ident) => ident,
                    _ => return Err(syn::Error::new_spanned(pat, "Argument de fonction attendu."))
                };
                let arg_name = &variable.ident;
                let arg_name_str = arg_name.to_string();
                let ty = match ty.as_ref() {
                    Type::Path(TypePath { path, .. }) => path,
                    Type::Reference(TypeReference { elem, .. }) => match elem.as_ref() {
                        Type::Path(TypePath { path, .. }) => path,
                        _ => return Err(syn::Error::new_spanned(ty, "Type d'argument innatendu."))
                    },
                    _ => return Err(syn::Error::new_spanned(ty, "Type d'argument innatendu."))
                };
                let ty_last = match ty.segments.last() {
                    Some(segment) => segment,
                    None => return Err(syn::Error::new_spanned(ty, "discord_argument: Erreur innatendu."))
                };
                let ty_name = ty_last.ident.to_string();
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
                            variable_name: quote! { #arg_name },
                            decode_expr: Some(quote! { let #arg_name =  #value_decoded.cloned(); }),
                            is_self: false
                        })
                    }
                    "ApplicationCommandEmbed" => {
                        Ok(Argument {
                            variable_name: quote! { &app_command },
                            decode_expr: None,
                            is_self: false
                        })
                    }
                    "Context" => {
                        Ok(Argument {
                            variable_name: quote! { &ctx },
                            decode_expr: None,
                            is_self: false
                        })
                    }
                    _ => {
                        let value_decoded = Self::argument_decode(&arg_name_str, ty)?;
                        let error_msg = format!("Argument \"{}\" manquant.", arg_name_str);
                        Ok(Argument {
                            variable_name: quote! { #arg_name },
                            decode_expr: Some(quote! { let #arg_name =  #value_decoded.ok_or_else(|| #error_msg).unwrap().to_owned(); }),
                            is_self: false
                        })
                    }
                }
            },
            v => Ok(Argument {
                variable_name: quote! { #v },
                decode_expr: None,
                is_self: true
            })
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
            "&str" | "str" => return Err(syn::Error::new_spanned(ty, "Utilisez String à la place.")),
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