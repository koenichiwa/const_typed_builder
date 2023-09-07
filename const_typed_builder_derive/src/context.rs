use core::fmt;
use quote::ToTokens;
use std::fmt::{Debug, Display};

/// A type to collect errors together and format them.
///
/// References can be shared since this type uses run-time exclusive mut checking.
#[derive(Debug, Default)]
pub struct Context {
    errors: Vec<syn::Error>,
}

impl Context {
    pub fn new() -> Self {
        Context { errors: Vec::new() }
    }

    /// Add an error to the context object with a tokenenizable object.
    ///
    /// The object is used for spanning in error messages.
    pub fn error_spanned_by<A: ToTokens, T: Display>(&mut self, obj: A, msg: T) {
        self.errors
            .push(syn::Error::new_spanned(obj.into_token_stream(), msg));
    }

    /// Add one of Syn's parse errors.
    pub fn syn_error(&mut self, err: syn::Error) {
        self.errors.push(err);
    }

    pub fn get_error(&self) -> Option<syn::Error> {
        self.errors
            .iter()
            .fold::<Option<syn::Error>, _>(None, |acc, err| {
                if let Some(mut acc) = acc {
                    acc.combine(err.clone());
                    Some(acc)
                } else {
                    Some(err.clone())
                }
            })
    }

    pub fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "const_builder_derive:\n");
        self.get_error().fmt(f)
    }
}
