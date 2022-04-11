// https://blog.turbo.fish/proc-macro-basics/#:~:text=Procedural%20macros%20%28often%20shortened%20to%20%22proc-macros%22%29%20are%20a,Rust%20code%20that%20is%20run%20at%20compile%20time.

use std::sync::Mutex;
use quote::quote;
use proc_macro::TokenStream;

lazy_static::lazy_static!(
    static ref TEST_COUNTER: Mutex<i32> = Mutex::new(0);
);

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
#[proc_macro_attribute]
pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
#[proc_macro_attribute]
pub fn commands(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_commands(item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}
#[derive(Debug, Clone)]
struct Function {
    signature: syn::Signature,
    body: syn::Block,
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
    
    let interfs = implement.items.iter().cloned().filter_map(|item| {
        match item {
            ImplItem::Method(ImplItemMethod { attrs, sig, block, .. }) => {
                if attrs.iter().any(|attr| attr.path.is_ident("command")) {
                    Some(ComponentInterface::Command{
                        function: Function{
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
                            signature: sig.clone(),
                            body: block.clone()
                        }

                    })
                } else {
                    None
                }
            },
            _ => Some(ComponentInterface::Other(item)),
        }
    });
    for interf in interfs {
        match interf {
            ComponentInterface::Command { function } => {
                
            },
            ComponentInterface::Event { event_name, function } => {
                todo!()
            },
            ComponentInterface::Other(item) => {
                todo!()
            }
        }
    }
    let events: Vec<Ident> = vec![];
    let commands: Vec<Ident> = vec![];
    let evt = quote! {
        match event {
            serenity::model::event::Event::InteractionCreate(interaction) => {
                let command_name = todo!();
                match command_name {
                    #(#commands), *
                }
            },
            #(#events), *
            _ => {}
        }
    };
    
    Ok(quote!{
        #implement
    }.into())
}