use proc_macro2::Span;
use syn::{spanned::Spanned, Attribute};

use crate::util::strip_raw_ident_prefix;

#[derive(Debug)]
pub struct FieldInfo<'a> {
    pub index: usize,
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
        field: &'a syn::Field,
        default_settings: &FieldSettings,
    ) -> Result<FieldInfo<'a>, syn::Error> {
        if let Some(ref name) = field.ident {
            let mut settings = default_settings.clone().with(&field.attrs);
            FieldInfo {
                index,
                name,
                ty: &field.ty,
                settings,
                // generic_ident: syn::Ident::new(&format!("__{}", strip_raw_ident_prefix(name.to_string())), Span::call_site()),
            }
            .post_process()
        } else {
            Err(syn::Error::new(field.span(), "Nameless field in struct"))
        }
    }

    pub fn is_option(&self) -> bool {
        if let syn::Type::Path(type_path) = self.ty {
            if type_path.qself.is_none() {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident == "Option"
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn inner_type(&self) -> Option<&syn::Type> {
        let path = if let syn::Type::Path(type_path) = self.ty {
            if type_path.qself.is_some() {
                return None;
            }
            &type_path.path
        } else {
            return None;
        };
        let segment = path.segments.last()?;
        let generic_params =
            if let syn::PathArguments::AngleBracketed(generic_params) = &segment.arguments {
                dbg!(generic_params)
            } else {
                return None;
            };
        if let syn::GenericArgument::Type(ty) = generic_params.args.first()? {
            Some(ty)
        } else {
            None
        }
    }

    fn post_process(mut self) -> Result<Self, syn::Error> {
        if !self.settings.mandatory {
            self.settings.mandatory = !self.is_option();
        }

        Ok(self)
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
    fn new() -> FieldSettings {
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
