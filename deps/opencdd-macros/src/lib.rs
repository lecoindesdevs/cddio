// https://blog.turbo.fish/proc-macro-basics/#:~:text=Procedural%20macros%20%28often%20shortened%20to%20%22proc-macros%22%29%20are%20a,Rust%20code%20that%20is%20run%20at%20compile%20time.

use std::sync::Mutex;
use quote::quote;
use proc_macro::TokenStream;
mod function;
use function::Function;

lazy_static::lazy_static!(
    static ref TEST_COUNTER: Mutex<i32> = Mutex::new(0);
);

// #[proc_macro_attribute]
// pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
// #[proc_macro_attribute]
// pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     item
// }
#[proc_macro_attribute]
pub fn commands(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_commands(item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}


#[derive(Debug, Clone)]
enum ComponentInterface {
    Command{
        function: Function
    },
    Event {
        event_name: syn::Ident,
        function: Function
    },
    Other(syn::ImplItem)
}

fn expand_commands(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    use syn::*;
    let implement: ItemImpl = match syn::parse2(input){
        Ok(item) => item,
        Err(e) => return Err(syn::Error::new(e.span(), "Implémentation d'une structure attendue."))
    };
    println!("{:#?}", implement.self_ty);
    let struct_name = match implement.self_ty.as_ref() {
        syn::Type::Path(v) => v,
        v => return Err(syn::Error::new_spanned(v, "Implémentation d'une structure attendue."))
    };
    // let test1 = vec![1,2];
    // let test2 = vec![3, 4];
    // let tok = quote! {
    //     #(#test1, #test2);*
    // };
    // println!("{}", tok);

    
    let interfs = implement.items.into_iter()
        .filter_map(|item| {
        let ImplItemMethod { attrs, sig, block, .. } = match item {
            ImplItem::Method(v) => v,
            item => return Some(ComponentInterface::Other(item)),
        };
            
        if attrs.iter().any(|attr| attr.path.is_ident("command")) {
            Some(ComponentInterface::Command{
                function: Function{
                    attributes: attrs.into_iter().filter(|attr| !attr.path.is_ident("command")).collect(),
                    signature: sig.clone(),
                    body: block.clone()
                }
            })
        } else if let Some(attr) = attrs.iter().find(|attr| attr.path.is_ident("event")) {
            let evt_name = match attr.parse_args::<Ident>() {
                Ok(item) => item,
                Err(_) => return None
            };
            Some(ComponentInterface::Event { 
                event_name: evt_name, 
                function: Function {
                    attributes: attrs.into_iter().filter(|attr| !attr.path.is_ident("event")).collect(),
                    signature: sig.clone(),
                    body: block.clone()
                }
            })
        } else {
            None
        }
    });
    let mut events: Vec<proc_macro2::TokenStream> = vec![];
    let mut commands: Vec<proc_macro2::TokenStream> = vec![];
    let mut impl_items: Vec<proc_macro2::TokenStream> = vec![];

    for interf in interfs {
        match interf {
            ComponentInterface::Command { function } => {
                let command_str = function.function_name().to_string();
                let func_call = function.function_call_event()?;
                println!("{}", func_call);
                let func_decl = function.function_decl();
                commands.push(quote! {
                    #command_str => {#func_call}
                });
                impl_items.push(quote! {
                    #func_decl
                });
            },
            ComponentInterface::Event { event_name, function } => {
                todo!()
            },
            ComponentInterface::Other(item) => {
                impl_items.push(quote!(#item));
            }
        }
    }
    let impl_event = quote! {
        impl Component2 for #struct_name {
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
    let impl_functions = quote! {
        impl #struct_name {
            #(#impl_items)*
        }
    };
    let result = quote! {
        #impl_event
        #impl_functions
    };
    println!("{0:=<30}\n{1: ^30}\n{0:=<30}\n{result:#}", "", "FINAL RESULT");
    Ok(result.into())
}