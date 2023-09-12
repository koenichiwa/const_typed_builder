use std::collections::HashMap;

use crate::{
    field_info::{FieldInfo, FieldSettings},
    group_info::{GroupInfo, GroupType},
    symbol::GROUP,
};

type FieldInfos<'a> = Vec<FieldInfo<'a>>;

#[derive(Debug)]
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
            let settings = StructSettings::new().with_attrs(attrs)?;
            let (field_infos, settings) = Self::parse_fields(settings, fields)?;

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
            Ok(info)
        } else {
            Err(syn::Error::new_spanned(ast, "Builder is only supported for named structs"))
        }
    }

    fn parse_fields(
        settings: StructSettings,
        fields: &'a syn::FieldsNamed,
    ) -> syn::Result<(FieldInfos<'a>, StructSettings)> {
        if fields.named.is_empty() {
            return Err(syn::Error::new_spanned(fields, "No fields found"));
        }
        fields
            .named
            .iter()
            .map(|field| FieldInfo::new( field, &settings))
            .collect::<syn::Result<Vec<_>>>()
            .map_or(Err(syn::Error::new_spanned(fields, "No fields found")), |infos| Self::fields_with_indices(infos, settings))
    }

    fn fields_with_indices(
        fields: FieldInfos<'a>,
        mut settings: StructSettings,
    ) -> syn::Result<(FieldInfos<'a>, StructSettings)> {
        let infos = fields
            .into_iter()
            .map(|mut info| {
                if !&info.group_names().is_empty() {
                    info.group_names().clone().iter().map(|group_name| {
                        let group = settings
                            .groups
                            .get_mut(group_name.as_str())
                            .ok_or(syn::Error::new_spanned(info.name(), format!("Group {} not found", group_name.as_str())))?;
                        info.set_group_index(group.clone(), group.member_count());
                        group.incr_member_count();
                        Ok(())
                    }).collect::<syn::Result<()>>()?;
                }

                if info.mandatory() {
                    info.set_mandatory_index(settings.mandatory_count);
                    settings.mandatory_count += 1;
                }
                Ok(info)
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok((infos, settings))
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

    pub fn mandatory_count(&self) -> usize {
        self.settings.mandatory_count
    }

    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.settings.groups
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

impl  Default for StructSettings {
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

    pub fn default_field_settings(&self) -> &FieldSettings {
        &self.default_field_settings
    }

    pub fn with_attrs(
        mut self,
        attrs: &Vec<syn::Attribute>,
    ) -> syn::Result<Self> {
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
