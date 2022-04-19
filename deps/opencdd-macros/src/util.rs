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