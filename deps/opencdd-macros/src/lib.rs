mod function;
mod command;
mod event;
mod message_component;

mod util;
mod log;
mod group;

use std::sync::Mutex;
use quote::quote;
use proc_macro::TokenStream;
use function::{Function, RefFunction, FunctionType};
use std::rc::Rc;

lazy_static::lazy_static!(
    static ref TEST_COUNTER: Mutex<i32> = Mutex::new(0);
);

#[proc_macro_attribute]
pub fn commands(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_commands(item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

enum MyImplItem {
    Function(RefFunction),
    Other(syn::ImplItem),
}

impl quote::ToTokens for MyImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MyImplItem::Function(f) => f.as_ref().borrow().to_tokens(tokens),
            MyImplItem::Other(i) => i.to_tokens(tokens),
        }
    }
}

fn expand_commands(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    use syn::*;
    let implement: ItemImpl = match syn::parse2(input){
        Ok(item) => item,
        Err(e) => return Err(syn::Error::new(e.span(), "Implémentation d'une structure attendue."))
    };
    let struct_name = match implement.self_ty.as_ref() {
        syn::Type::Path(v) => v,
        v => return Err(syn::Error::new_spanned(v, "Implémentation d'une structure attendue."))
    };
    let (attrs_group, attrs): (Vec<_>, Vec<_>) = implement.attrs.into_iter().partition(|attr| attr.path.is_ident("group"));
    let mut groups = self::group::GroupManager::from_iter(attrs_group.into_iter())?;
    
    let interfs = implement.items.into_iter()
        .map(|item| -> syn::Result<_> {
            match item {
                ImplItem::Method(v) => {
                    let function = FunctionType::new_rc(v)?;
                    Ok(MyImplItem::Function(function))
                },
                item => Ok(MyImplItem::Other(item)),
            }
        });
    let mut events: Vec<proc_macro2::TokenStream> = vec![];
    let mut commands: Vec<proc_macro2::TokenStream> = vec![];
    let mut impl_items: Vec<proc_macro2::TokenStream> = vec![];

    for interf in interfs {
        let interf = interf?;
        let func_rc = match interf {
            MyImplItem::Function(f) => f,
            MyImplItem::Other(other) => {
                impl_items.push(quote! { #other });
                continue;
            }
        };
        let func = func_rc.borrow();
        match &*func {
            FunctionType::Event(event) => {
                events.push(event.event_handle()?);
                impl_items.push(quote! {
                    #event
                });
            },
            FunctionType::Command(command) => {
                
                let event = command.event_handle()?;
                let name = command.attr.name.clone().or_else(|| Some(command.name().to_string())).unwrap();
                impl_items.push(quote! {
                    #command
                });
                let name = if let Some(grp) = &command.attr.group {
                    let group_found = match groups.find_group(&grp) {
                        Some(group) => group,
                        None => return Err(syn::Error::new_spanned(&grp, "Groupe introuvable."))
                    };
                    group_found.borrow_mut().add_function(Rc::clone(&func_rc));
                    format!("{}.{}",group_found.borrow().get_fullname(), name)
                } else {
                    groups.root_mut().add_function(Rc::clone(&func_rc));
                    name
                };
                commands.push(quote! {
                    #name => {#event}
                });
            },
            FunctionType::NoSpecial(v) => {
                impl_items.push(quote! { #v });
            },
        }
    }
    let impl_event = quote! {
        #[serenity::async_trait]
        impl opencdd_components::ComponentEvent for #struct_name {
            async fn event(&self, ctx: &serenity::client::Context, event: &serenity::model::event::Event) {
                match event {
                    serenity::model::event::Event::InteractionCreate(serenity::model::event::InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(orig_app_command), ..}) => {
                        let app_command = opencdd_components::ApplicationCommandEmbed::new(orig_app_command);
                        let command_name = app_command.fullname();
                        match command_name.as_str() {
                            #(#commands), *
                            _ => ()
                        }
                    },
                    #(#events,)*
                    _ => ()
                }
            }
        }
    };
    let impl_functions = quote! {
        #(#attrs)*
        impl #struct_name {
            #(#impl_items)*
        }
    };
    let declaratives = groups.get_declarative();
    let impl_declaratives = quote!{
        impl opencdd_components::ComponentDeclarative for #struct_name {
            fn declarative(&self) -> Option<&'static opencdd_components::declarative::Node> {
                const decl: opencdd_components::declarative::Node = #declaratives;
                Some(&decl)
            }
        }
    };
    let result = quote! {
        #impl_event
        #impl_declaratives

        impl opencdd_components::Component for #struct_name {}
        
        #impl_functions
    };
    log::log(&format_args!("{0:=<30}\n{1: ^30}\n{0:=<30}\n{result:#}", "", "FINAL RESULT"));
    Ok(result.into())
}