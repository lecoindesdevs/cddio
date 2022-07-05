use proc_macro2::TokenStream;

pub trait FindAndPop<T> {
    /// Finds the first element that satisfies the given predicate,
    /// and removes it from the container.
    fn find_and_pop<C>(self, mut predicate: impl FnMut(&T) -> bool) -> (Option<T>, C) 
        where Self: Sized + IntoIterator<Item = T>,
        C: Default + FromIterator<T>
    {
        let mut found = None;
        let col = self.into_iter().filter_map(|v| {
            if predicate(&v) && found.is_none() {
                found = Some(v);
                None
            } else {
                Some(v)
            }
        }).collect::<C>();
        (found, col)
    }
}

impl<I, T> FindAndPop<T> for I where I: IntoIterator<Item = T>  {}

/// Something arounded by parenthesis: `(token)`
pub struct ParenValue<T> 
    where T: syn::parse::Parse
{
    pub _paren: syn::token::Paren,
    pub value: T,
}
impl<T: syn::parse::Parse> syn::parse::Parse for ParenValue<T>  {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let paren = syn::parenthesized!(content in input);
        let value = content.parse()?;
        Ok(ParenValue {
            _paren: paren,
            value,
        })
    }
}

/// Argument like `a=2`
pub struct MacroArg {
    pub name: syn::Ident,
    pub eq: syn::Token![=],
    pub value: syn::Lit,
}
impl syn::parse::Parse for MacroArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let eq = input.parse()?;
        let value = input.parse()?;
        Ok(Self { name, eq, value })
    }
}
impl quote::ToTokens for MacroArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        self.eq.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}
/// Arguments like `a=2, b=3`
pub struct MacroArgs {
    pub args: syn::punctuated::Punctuated<MacroArg, syn::Token![,]>,
}
impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = input.parse_terminated(MacroArg::parse)?;
        Ok(Self { args })
    }
}

pub fn fn_args_to_args_call(fn_args: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>) -> syn::Result<TokenStream> {
    use syn::*;
    use quote::quote;
    let mut args = TokenStream::new();
    for arg in fn_args {
        match arg {
            FnArg::Receiver(_) => continue,
            FnArg::Typed(arg) => {
                match arg.pat.as_ref() {
                    Pat::Ident(ident) => args.extend(quote! { #ident, }),
                    Pat::Wild(_) => args.extend(quote! { _, }),
                    _ => return Err(syn::Error::new_spanned(&arg.pat, "Unsupported pattern"))
                }
            }
        }
    }
    Ok(args)
}