mod function;
mod argument;
mod util;
mod log;
mod group;

use std::sync::Mutex;
use quote::quote;
use proc_macro::TokenStream;
use function::{Function, FunctionType};

lazy_static::lazy_static!(
    static ref TEST_COUNTER: Mutex<i32> = Mutex::new(0);
);

#[proc_macro_attribute]
pub fn commands(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_commands(item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}

enum MyImplItem {
    Command(Function),
    Other(syn::ImplItem),
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
    let groups = self::group::GroupManager::from_iter(attrs_group.into_iter())?;
    log::log(&format_args!("{:#?}", groups));
    let interfs = implement.items.into_iter()
        .map(|item| -> syn::Result<_> {
            match item {
            ImplItem::Method(v) => {
                let function = Function::new(v)?;
                Ok(MyImplItem::Command(function))
            },
            item => Ok(MyImplItem::Other(item)),
            }
    });
    let mut events: Vec<proc_macro2::TokenStream> = vec![];
    let mut commands: Vec<proc_macro2::TokenStream> = vec![];
    let mut declaratives: Vec<proc_macro2::TokenStream> = vec![];
    let mut impl_items: Vec<proc_macro2::TokenStream> = vec![];

    for interf in interfs {
        let interf = interf?;
        match interf {
            MyImplItem::Command(function @  Function{ fn_type: FunctionType::Command(_), .. }) => {
                let command_str = function.command_name();
                let func_call = function.function_call_event()?;
                commands.push(quote! {
                    #command_str => {#func_call}
                });
                impl_items.push(quote! {
                    #function
                });
                // let decl = function.get_declarative();
                // declaratives.push(quote! {
                //     #decl
                // });
            },
            MyImplItem::Command(function @ Function{ fn_type: FunctionType::Event, .. }) => {
                todo!()
            },
            MyImplItem::Command(function) => {
                impl_items.push(quote!(#function));
            },
            MyImplItem::Other(item) => {
                impl_items.push(quote!(#item));
            }
        }
    }
    let impl_event = quote! {
        impl ComponentEvent for #struct_name {
            fn event(&mut self, ctx: &serenity::client::Context, event: &serenity::model::event::Event) {
                match event {
                    serenity::model::event::Event::InteractionCreate(serenity::model::event::InteractionCreateEvent{interaction: serenity::model::interactions::Interaction::ApplicationCommand(orig_app_command), ..}) => {
                        let app_command = super::utils::app_command::ApplicationCommandEmbed::new(orig_app_command);
                        let command_name = app_command.fullname();
                        match command_name.as_str() {
                            #(#commands), *
                            _ => ()
                        }
                    },
                    #(#events), *
                    _ => ()
                }
            }
        }
    };
    let impl_declarative = quote! {
        impl ComponentDeclarative for #struct_name {
            fn declarative(&self) -> &'static [Command] {
                &[
                    #(#declaratives), *
                ]
            }
        }
    };
    let impl_functions = quote! {
        #(#attrs)*
        impl #struct_name {
            #(#impl_items)*
        }
    };
    let result = quote! {
        #impl_event
        #impl_functions
        #impl_declarative
    };
    log::log(&format_args!("{0:=<30}\n{1: ^30}\n{0:=<30}\n{result:#}", "", "FINAL RESULT"));
    Ok(result.into())
}