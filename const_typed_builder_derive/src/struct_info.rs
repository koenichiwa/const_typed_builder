use std::{borrow::Cow, collections::BTreeSet};

use crate::{
    context::Context,
    field_info::{FieldInfo, FieldSettings},
};

type FieldInfos<'a> = Vec<FieldInfo<'a>>;

pub struct StructInfo<'a> {
    input: &'a syn::DeriveInput,
    attrs: &'a Vec<syn::Attribute>,
    vis: &'a syn::Visibility,
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
    fields: &'a syn::FieldsNamed,
    field_infos: FieldInfos<'a>,
    settings: StructSettings,
}

impl<'a> StructInfo<'a> {
    pub fn new(context: &mut Context, ast: &'a syn::DeriveInput) -> Option<Self> {
        if let syn::DeriveInput {
            attrs,
            vis,
            ident,
            generics,
            data:
                syn::Data::Struct(syn::DataStruct {
                    fields: syn::Fields::Named(fields),
                    ..
                }),
        } = &ast
        {
            let settings = StructSettings::new();
            let field_infos = Self::parse_fields(context, &settings, fields)?;

            let info = StructInfo {
                input: ast,
                attrs,
                vis,
                ident,
                generics,
                fields,
                field_infos,
                settings,
            };
            Some(info)
        } else {
            context.error_spanned_by(ast, "Builder is only supported for named structs");
            None
        }
    }

    fn parse_fields(
        context: &mut Context,
        settings: &StructSettings,
        fields: &'a syn::FieldsNamed,
    ) -> Option<FieldInfos<'a>> {
        if fields.named.is_empty() {
            context.error_spanned_by(fields, "No fields found");
        }

        let mut mandatory_index = 0;

        fields
            .named
            .iter()
            .enumerate()
            .map(|(index, field)| {
                FieldInfo::new(context, index, mandatory_index, field, settings).map(|info| {
                    if info.mandatory() {
                        mandatory_index += 1;
                    }
                    info
                })
            })
            .collect::<Option<Vec<_>>>()
    }

    pub fn name(&self) -> &syn::Ident {
        self.ident
    }

    pub fn builder_name(&self) -> syn::Ident {
        quote::format_ident!("{}{}", self.ident, self.settings.builder_suffix)
    }

    pub fn data_name(&self) -> syn::Ident {
        quote::format_ident!("{}{}", self.ident, self.settings.data_suffix)
    }

    pub fn field_infos(&self) -> &FieldInfos {
        &self.field_infos
    }

    pub fn mandatory_identifiers(&self) -> BTreeSet<syn::Ident> {
        self.field_infos
            .iter()
            .filter_map(|field| field.mandatory_ident())
            .collect()
    }
}

pub struct StructSettings {
    builder_suffix: String,
    data_suffix: String,
    default_field_settings: FieldSettings,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            builder_suffix: "Builder".to_string(),
            data_suffix: "Data".to_string(),
            default_field_settings: FieldSettings::new(),
        }
    }
}

impl StructSettings {
    fn new() -> Self {
        Default::default()
    }

    pub fn default_field_settings(&self) -> &FieldSettings {
        &self.default_field_settings
    }
}
