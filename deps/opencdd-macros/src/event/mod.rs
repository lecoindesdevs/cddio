use crate::util::ParenValue;
use std::fmt;
#[derive(Debug, Clone)]
pub struct EventAttribute {
    pub name: String,
}

impl EventAttribute {
    fn from_attr(attr: syn::Attribute) -> syn::Result<Self> {
        use syn::*;
        
        let arg_span = attr.span();
        let args = parse2::<ParenValue<Ident>>(attr.tokens)?;
        Ok(EventAttribute{
            name: args.value.to_string()
        })
    }
}

pub struct Event {
    attr: EventAttribute
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
                    .field("event", &self.attr.name)
                    .finish()
    }
}