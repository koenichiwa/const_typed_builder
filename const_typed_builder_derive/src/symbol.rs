use proc_macro2::Span;
use std::fmt::Display;
use syn::{Ident, Path};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Symbol<'a>(&'a str);

pub const MANDATORY: Symbol = Symbol("mandatory");
pub const GROUP: Symbol = Symbol("group");
pub const BUILDER: Symbol = Symbol("builder");
pub const SINGLE: Symbol = Symbol("single");
pub const AT_LEAST: Symbol = Symbol("at_least");
pub const AT_MOST: Symbol = Symbol("at_most");
pub const EXACT: Symbol = Symbol("exact");
pub const PROPAGATE: Symbol = Symbol("propagate");

impl<'a> From<&'a String> for Symbol<'a> {
    fn from(value: &'a String) -> Self {
        Symbol(value)
    }
}

impl<'a> From<Symbol<'a>> for syn::Ident {
    fn from(value: Symbol) -> Self {
        syn::Ident::new(value.0, Span::call_site())
    }
}

// impl <'a> From<&'a Ident> for Symbol<'a> {
//     fn from(value: &'a Ident) -> Self {
//         Symbol(value.to_string().as_str())
//     }
// }

impl<'a> PartialEq<Symbol<'a>> for String {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol<'a>> for str {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol<'a>> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol<'a>> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl<'a> PartialEq<Symbol<'a>> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol<'a>> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> Display for Symbol<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}
