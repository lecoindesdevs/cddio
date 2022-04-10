// https://blog.turbo.fish/proc-macro-basics/#:~:text=Procedural%20macros%20%28often%20shortened%20to%20%22proc-macros%22%29%20are%20a,Rust%20code%20that%20is%20run%20at%20compile%20time.

use std::sync::Mutex;
use quote::quote;
use proc_macro::TokenStream;

lazy_static::lazy_static!(
    static ref TEST_COUNTER: Mutex<i32> = Mutex::new(0);
);

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    // println!("item: \"{:?}\"", item);
    let mut counter = TEST_COUNTER.lock().unwrap();
    *counter += 1;
    println!("(command) counter: {}", counter);
    item
}
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());
    // println!("item: \"{:?}\"", item);
    let mut counter = TEST_COUNTER.lock().unwrap();
    *counter += 1;
    println!("(component) counter: {}", counter);
    item
}
#[proc_macro_attribute]
pub fn commands(attr: TokenStream, item: TokenStream) -> TokenStream {
    // println!("attr: \"{}\"", attr.to_string());

    expand_commands(item.into()).unwrap_or_else(syn::Error::into_compile_error).into()

    // println!("item: \"{:?}\"", item);
    // let mut counter = TEST_COUNTER.lock().unwrap();
    // *counter += 1;
    // println!("(commands) counter: {}", counter);
    // item
}



fn expand_commands(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    use syn::*;
    let implement: ItemImpl = match syn::parse2(input){
        Ok(item) => item,
        Err(e) => return Err(syn::Error::new(e.span(), "Implémentation d'une structure attendue."))
    };
    
    let commands = implement.items.iter().filter_map(|item| {
        match item {
            ImplItem::Method(ImplItemMethod { attrs, sig: Signature{ident, ..}, .. }) => {
                if attrs.iter().any(|attr| attr.path.is_ident("command")) {
                    Some(ident.clone())
                } else {
                    None
                }
            },
            _ => None,
        }
    });
    for command in commands {
        println!("command: {}", command);
    }
    
    //todo!()
    Ok(quote!{
        #implement
    }.into())
}