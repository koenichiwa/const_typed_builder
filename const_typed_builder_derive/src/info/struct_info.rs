use std::collections::HashMap;

use super::field_info::{FieldInfo, FieldSettings};
use super::group_info::{GroupInfo, GroupType};
use quote::format_ident;

use crate::symbol::GROUP;

type FieldInfos<'a> = Vec<FieldInfo<'a>>;

#[derive(Debug)]
pub struct StructInfo<'a> {
    input: &'a syn::DeriveInput,
    ident: &'a syn::Ident,
    builder_ident: syn::Ident,
    data_ident: syn::Ident,
    groups: HashMap<String, GroupInfo>,
    field_infos: FieldInfos<'a>,
}

impl<'a> StructInfo<'a> {
    pub fn new(ast: &'a syn::DeriveInput) -> syn::Result<Self> {
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
            let mut settings = StructSettings::new().with_attrs(attrs)?;
            let field_infos = Self::parse_fields(&mut settings, fields)?;

            let info = StructInfo {
                input: ast,
                ident,
                builder_ident: format_ident!("{}{}", ident, settings.builder_suffix),
                data_ident: format_ident!("{}{}", ident, settings.data_suffix),
                groups: settings.groups,
                field_infos,
            };
            Ok(info)
        } else {
            Err(syn::Error::new_spanned(
                ast,
                "Builder is only supported for named structs",
            ))
        }
    }

    fn parse_fields(
        settings: &mut StructSettings,
        fields: &'a syn::FieldsNamed,
    ) -> syn::Result<FieldInfos<'a>> {
        if fields.named.is_empty() {
            return Err(syn::Error::new_spanned(fields, "No fields found"));
        }
        fields
            .named
            .iter()
            .map(|field| FieldInfo::new(field, settings))
            .collect::<syn::Result<Vec<_>>>()
    }

    pub fn name(&self) -> &syn::Ident {
        self.ident
    }

    pub fn builder_name(&self) -> &syn::Ident {
        &self.builder_ident
    }

    pub fn data_name(&self) -> &syn::Ident {
        &self.data_ident
    }

    pub fn field_infos(&self) -> &FieldInfos {
        &self.field_infos
    }

    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.groups
    }
}

#[derive(Debug)]
pub struct StructSettings {
    builder_suffix: String,
    data_suffix: String,
    default_field_settings: FieldSettings,
    groups: HashMap<String, GroupInfo>,
    mandatory_count: usize,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            builder_suffix: "Builder".to_string(),
            data_suffix: "Data".to_string(),
            default_field_settings: FieldSettings::new(),
            groups: HashMap::new(),
            mandatory_count: 0,
        }
    }
}

impl StructSettings {
    fn new() -> Self {
        Default::default()
    }

    pub fn next_mandatory(&mut self) -> usize {
        self.mandatory_count += 1;
        self.mandatory_count - 1
    }

    pub fn next_group_index(&mut self, group_name: &String) -> Option<usize> {
        let res = self.groups.get_mut(group_name)?.next_index();
        Some(res)
    }

    pub fn group_by_name(&self, group_name: &String) -> Option<&GroupInfo> {
        self.groups.get(group_name)
    }

    pub fn default_field_settings(&self) -> &FieldSettings {
        &self.default_field_settings
    }

    pub fn with_attrs(mut self, attrs: &Vec<syn::Attribute>) -> syn::Result<Self> {
        attrs
            .iter()
            .map(|attr| self.handle_attribute(attr))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self)
    }

    fn handle_attribute(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if let Some(ident) = attr.path().get_ident() {
            if ident == GROUP {
                self.handle_group_attribute(attr)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn handle_group_attribute(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        let list = attr.meta.require_list()?;
        if list.tokens.is_empty() {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            let group_name = meta
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&attr.meta, "Can't parse group name"))?
                .clone();

            let expr: syn::Expr = meta.value()?.parse()?;

            let group_type = match &expr {
                syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                    let group_type = if let syn::Expr::Path(syn::ExprPath { path, .. }) =
                        func.as_ref()
                    {
                        path.get_ident()
                            .ok_or_else(|| syn::Error::new_spanned(&func, "Can't parse group type"))
                    } else {
                        Err(syn::Error::new_spanned(&func, "Can't find group type"))
                    }?;

                    let group_args = if let Some(syn::Expr::Lit(syn::ExprLit {
                        attrs,
                        lit: syn::Lit::Int(val),
                    })) = args.first()
                    {
                        val.base10_parse::<usize>()
                    } else {
                        Err(syn::Error::new_spanned(&func, "Can't parse group args"))
                    }?;

                    match group_type.to_string().as_str() {
                        "exact" => Ok(GroupType::Exact(group_args)),
                        "at_least" => Ok(GroupType::AtLeast(group_args)),
                        "at_most" => Ok(GroupType::AtMost(group_args)),
                        "single" => Err(syn::Error::new_spanned(
                            args,
                            "`single` doesn't take any arguments",
                        )),
                        _ => Err(syn::Error::new_spanned(group_type, "Unknown group type")),
                    }
                }
                syn::Expr::Path(syn::ExprPath { path, .. }) => {
                    let group_type = path
                        .get_ident()
                        .ok_or_else(|| syn::Error::new_spanned(&path, "Can't parse group type"))?;
                    match group_type.to_string().as_str() {
                        "exact" | "at_least" | "at_most" => Err(syn::Error::new_spanned(
                            &attr.meta,
                            "Missing arguments for group type",
                        )),
                        "single" => Ok(GroupType::Exact(1)),
                        _ => Err(syn::Error::new_spanned(&attr.meta, "Can't parse group")),
                    }
                }
                _ => Err(syn::Error::new_spanned(&attr.meta, "Can't parse group")),
            };

            self.groups.insert(
                group_name.to_string(),
                GroupInfo::new(group_name, group_type?),
            );
            Ok(())
        })
    }
}
