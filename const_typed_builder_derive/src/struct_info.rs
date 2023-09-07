use std::collections::{BTreeSet, HashSet};

use syn::Token;

use crate::{
    field_info::{FieldInfo, FieldSettings},
    symbol::GROUP,
};

pub struct StructInfo<'a> {
    pub name: &'a syn::Ident,
    pub field_infos: Vec<FieldInfo<'a>>,
    pub settings: StructSettings,
}

impl<'a> StructInfo<'a> {
    pub fn new(
        ast: &'a syn::DeriveInput,
        fields: &'a syn::FieldsNamed,
    ) -> Result<StructInfo<'a>, syn::Error> {
        let settings = StructSettings::new();
        let mut mandatory_index = 0;
        let field_infos = fields
            .named
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field = FieldInfo::new(
                    index,
                    mandatory_index,
                    field,
                    &settings.default_field_settings,
                )?;
                if field.mandatory_index.is_some() {
                    mandatory_index += 1;
                }
                Ok(field)
            })
            .collect::<Result<Vec<FieldInfo>, syn::Error>>()?;
        StructInfo {
            name: &ast.ident,
            field_infos,
            settings,
        }
        .post_process(ast)
    }

    fn post_process(mut self, ast: &'a syn::DeriveInput) -> syn::Result<Self> {
        ast.attrs
            .iter()
            .map(|attr| self.handle_attribute(attr))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(self)
    }

    fn handle_attribute(&mut self, attr: &syn::Attribute) -> Result<(), syn::Error> {
        if let Some(ident) = attr.path().get_ident() {
            if ident != "builder" {
                return Ok(())
            }
        }
        if let syn::Meta::List(list) = &attr.meta {
            if list.tokens.is_empty() {
                return Ok(());
            }
        }

        attr.parse_nested_meta(|meta| {
            // if meta.path == GROUP {
            //     dbg!(&meta.path);
            //     if meta.input.peek(Token![=]) {
            //         let value = meta.value()?;
            //         // if let syn::Expr::Lit(syn::ExprLit {
            //         //     lit: syn::Lit::Str(lit),
            //         //     ..
            //         // }) = &expr
            //         // {
            //         //     if !self.settings.groups.insert(lit.value()) {
            //         //         return Err(syn::Error::new_spanned(
            //         //             &expr,
            //         //             "Multiple adds to the same group",
            //         //         ));
            //         //     }
            //         // }
            //         value;
            //     }
            // }
            Ok(())
        })
    }

    pub fn mandatory_identifiers(&self) -> BTreeSet<syn::Ident> {
        self.field_infos
            .iter()
            .filter_map(|field| field.mandatory_ident())
            .collect()
    }
}

pub struct StructSettings {
    default_field_settings: FieldSettings,
    available_groups: HashSet<String>,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            default_field_settings: FieldSettings::new(),
            available_groups: HashSet::new(),
        }
    }
}

impl StructSettings {
    fn new() -> StructSettings {
        Default::default()
    }
}
