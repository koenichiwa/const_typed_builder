use std::collections::HashSet;

use proc_macro2::Span;
use quote::format_ident;
use syn::{Token, ExprPath};

use crate::{
    symbol::{GROUP, MANDATORY},
    util::{inner_type, is_option},
    MANDATORY_PREFIX,
};

#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub index: usize,
    pub mandatory_index: Option<usize>,
    pub name: &'a syn::Ident,
    pub ty: &'a syn::Type,
    pub settings: FieldSettings,
    // generic_ident: syn::Ident,
}

#[derive(Debug, Clone)]
pub struct FieldSettings {
    pub mandatory: bool,
    pub input_name: syn::Ident,
    pub groups: HashSet<String>,
}

impl<'a> FieldInfo<'a> {
    pub fn new(
        index: usize,
        mandatory_index: usize,
        field: &'a syn::Field,
        default_settings: &FieldSettings,
    ) -> Result<FieldInfo<'a>, syn::Error> {
        if let Some(ref name) = field.ident {
            let settings = default_settings.clone().with(&field.attrs);
            FieldInfo {
                index,
                mandatory_index: None,
                name,
                ty: &field.ty,
                settings,
                // generic_ident: syn::Ident::new(&format!("__{}", strip_raw_ident_prefix(name.to_string())), Span::call_site()),
            }
            .post_process(field, mandatory_index)
        } else {
            Err(syn::Error::new_spanned(field, "Nameless field in struct"))
        }
    }

    fn handle_attribute(&mut self, attr: &syn::Attribute) -> Result<(), syn::Error> {
        if let Some(ident) = attr.path().get_ident() {
            if ident != "builder" {
                return Ok(());
            }
        }
        if let syn::Meta::List(list) = &attr.meta {
            if list.tokens.is_empty() {
                return Ok(());
            }
        }

        attr.parse_nested_meta(|meta| {
            if meta.path == MANDATORY {
                if meta.input.peek(Token![=]) {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(syn::LitBool { value, .. }),
                        ..
                    }) = expr
                    {
                        self.settings.mandatory = value;
                    }
                } else {
                    self.settings.mandatory = true;
                }
            }
            // if meta.path == GROUP {
            //     if meta.input.peek(Token![=]) {
            //         let expr: syn::Expr = meta.value()?.parse()?;
            //         if let syn::Expr::Path(ExprPath {path, ..}) = &expr {
            //             let group_name = path.get_ident().ok_or(syn::Error::new_spanned(&path, "Can't parse group"))?;
            //             if !self.settings.groups.insert(dbg!(group_name.to_string())) {
            //                 return Err(syn::Error::new_spanned(
            //                     &expr,
            //                     "Multiple adds to the same group",
            //                 ));
            //             }
            //         }
            //         if let syn::Expr::Lit(syn::ExprLit {
            //             lit: syn::Lit::Str(lit),
            //             ..
            //         }) = &expr
            //         {
            //             if !self.settings.groups.insert(lit.value()) {
            //                 return Err(syn::Error::new_spanned(
            //                     &expr,
            //                     "Multiple adds to the same group",
            //                 ));
            //             }
            //         }
            //     }
            // }
            Ok(())
        })
    }

    fn post_process(
        mut self,
        field: &'a syn::Field,
        mandatory_index: usize,
    ) -> Result<Self, syn::Error> {
        field
            .attrs
            .iter()
            .map(|attr| self.handle_attribute(attr))
            .collect::<Result<Vec<_>, _>>()?;

        if !self.settings.mandatory {
            self.settings.mandatory = !is_option(self.ty);
        }

        if self.settings.mandatory {
            self.mandatory_index = Some(mandatory_index)
        }

        Ok(self)
    }

    pub fn mandatory_status(&self) -> Result<MandatoryStatus, syn::Error> {
        let inner_type_error = || syn::Error::new_spanned(self.ty, "Cannot read inner type");

        match (self.settings.mandatory, is_option(self.ty)) {
            (true, true) => Ok(MandatoryStatus::MandatoryOption(
                inner_type(self.ty).ok_or_else(inner_type_error)?,
            )),
            (true, false) => Ok(MandatoryStatus::Mandatory),
            (false, true) => Ok(MandatoryStatus::Optional(
                inner_type(self.ty).ok_or_else(inner_type_error)?,
            )),
            (false, false) => unreachable!("Non-optional types are always mandatory"),
        }
    }

    pub fn mandatory_ident(&self) -> Option<syn::Ident> {
        self.mandatory_index
            .map(|idx| format_ident!("{}_{}", MANDATORY_PREFIX, idx))
    }
}

impl Default for FieldSettings {
    fn default() -> FieldSettings {
        FieldSettings {
            mandatory: false,
            input_name: syn::Ident::new("input", Span::call_site()),
            groups: HashSet::new(),
        }
    }
}

impl FieldSettings {
    pub fn new() -> FieldSettings {
        Self::default()
    }

    fn with(self, attrs: &[syn::Attribute]) -> FieldSettings {
        attrs.iter().for_each(|attr| {
            match &attr.meta {
                syn::Meta::Path(path) => println!("{path:?}"),
                syn::Meta::List(list) => println!("{list:?}"),
                syn::Meta::NameValue(n_value) => println!("{n_value:?}"),
            };
        });
        self
    }
}

#[derive(Debug)]
pub enum MandatoryStatus<'a> {
    Mandatory,
    MandatoryOption(&'a syn::Type),
    Optional(&'a syn::Type),
}
