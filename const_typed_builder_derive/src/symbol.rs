use proc_macro2::Span;
use std::fmt::Display;
use syn::{Ident, Path};

/// A `Symbol` is a wrapper around a string identifier used for constants and identifiers.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Symbol<'a>(&'a str);

/// Constant representing the "mandatory" symbol.
pub const MANDATORY: Symbol = Symbol("mandatory");
/// Constant representing the "group" symbol.
pub const GROUP: Symbol = Symbol("group");
/// Constant representing the "builder" symbol.
pub const BUILDER: Symbol = Symbol("builder");
/// Constant representing the "single" symbol.
pub const SINGLE: Symbol = Symbol("single");
/// Constant representing the "at_least" symbol.
pub const AT_LEAST: Symbol = Symbol("at_least");
/// Constant representing the "at_most" symbol.
pub const AT_MOST: Symbol = Symbol("at_most");
/// Constant representing the "exact" symbol.
pub const EXACT: Symbol = Symbol("exact");
/// Constant representing the "propagate" symbol.
pub const PROPAGATE: Symbol = Symbol("propagate");
/// Constant representing the "assume_mandatory" symbol.
pub const ASSUME_MANDATORY: Symbol = Symbol("assume_mandatory");
/// Constant representing the "optional" symbol.
pub const OPTIONAL: Symbol = Symbol("optional");

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
