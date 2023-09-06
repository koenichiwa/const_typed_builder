use std::collections::BTreeSet;

use crate::field_info::{FieldInfo, FieldSettings};

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
        Ok(StructInfo {
            name: &ast.ident,
            field_infos,
            settings,
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
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            default_field_settings: FieldSettings::new(),
        }
    }
}

impl StructSettings {
    fn new() -> StructSettings {
        Default::default()
    }
}
