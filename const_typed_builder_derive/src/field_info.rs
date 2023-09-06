use proc_macro2::Span;
use quote::format_ident;
use syn::Attribute;

use crate::{
    util::{inner_type, is_option},
    MANDATORY_NAME,
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

    fn handle_attribute(&mut self, attr: &Attribute) -> Result<(), syn::Error> {
        match &attr.meta {
            syn::Meta::Path(path) => {
                let ident = &path
                    .segments
                    .first()
                    .ok_or(syn::Error::new_spanned(&path, "Can't parse attribute"))?
                    .ident;
                match ident.to_string().as_str() {
                    "mandatory" => self.settings.mandatory = true,
                    _ => return Err(syn::Error::new(ident.span(), "Unknown attribute")),
                }
            }
            syn::Meta::List(_) => (),
            syn::Meta::NameValue(_) => (),
        }
        Ok(())
    }

    fn post_process(
        mut self,
        field: &'a syn::Field,
        mandatory_index: usize,
    ) -> Result<Self, syn::Error> {
        field
            .attrs
            .iter()
            .map(|attr| self.handle_attribute(dbg!(attr)))
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
        match (self.settings.mandatory, is_option(self.ty)) {
            (true, true) => Ok(MandatoryStatus::MandatoryOption(
                inner_type(self.ty)
                    .ok_or(syn::Error::new_spanned(self.ty, "Cannot read inner type"))?,
            )),
            (true, false) => Ok(MandatoryStatus::Mandatory),
            (false, true) => Ok(MandatoryStatus::Optional(
                inner_type(self.ty)
                    .ok_or(syn::Error::new_spanned(self.ty, "Cannot read inner type"))?,
            )),
            (false, false) => unreachable!("Non-optional types are always mandatory"),
        }
    }

    pub fn mandatory_ident(&self) -> Option<syn::Ident> {
        self.mandatory_index
            .map(|idx| format_ident!("{}_{}", MANDATORY_NAME, idx))
    }
}

impl Default for FieldSettings {
    fn default() -> FieldSettings {
        FieldSettings {
            mandatory: false,
            input_name: syn::Ident::new("input", Span::call_site()),
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
