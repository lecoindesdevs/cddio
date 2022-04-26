use syn::spanned::Spanned;
use crate::util::*;
#[derive(Debug, Clone)]
pub struct ArgumentAttribute {
    pub name: Option<String>,
    pub description: String,
}

impl ArgumentAttribute {
    pub fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        let arg_span = attr.span();
        let args = syn::parse2::<ParenValue<MacroArgs>>(attr.tokens)?;
        for arg in args.value.args.into_iter() {
            match (arg.name.to_string().as_str(), arg.value) {
                ("name", syn::Lit::Str(s)) => name = Some(s.value()),
                ("description", syn::Lit::Str(s)) => description = Some(s.value()),
                _ => return Err(syn::Error::new_spanned(arg.name, "Argument inconnu.")),
            }
        }
        if description.is_none() {
            return Err(syn::Error::new(arg_span, "missing description argument"));
        }
        Ok(ArgumentAttribute {
            name,
            description: description.unwrap(),
        })
    }
}