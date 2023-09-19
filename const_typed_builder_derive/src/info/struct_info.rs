use std::collections::{HashMap, HashSet};

use super::field_info::{FieldInfo, FieldSettings};
use super::group_info::{GroupInfo, GroupType};
use quote::format_ident;
use syn::Token;

use crate::symbol::{ASSUME_MANDATORY, AT_LEAST, AT_MOST, BUILDER, EXACT, GROUP, SINGLE};

/// A type alias for a collection of `FieldInfo` instances.
type FieldInfos<'a> = Vec<FieldInfo<'a>>;

/// Represents the information about a struct used for code generation.
#[derive(Debug)]
pub struct StructInfo<'a> {
    /// The identifier of the struct.
    ident: &'a syn::Ident,
    /// The visibility of the struct.
    vis: &'a syn::Visibility,
    /// The generics of the struct.
    generics: &'a syn::Generics,
    /// The identifier of the generated builder struct.
    builder_ident: syn::Ident,
    /// The identifier of the generated data struct.
    data_ident: syn::Ident,
    _mandatory_indices: HashSet<usize>,
    /// A map of group names to their respective `GroupInfo`.
    groups: HashMap<String, GroupInfo>,
    /// A collection of `FieldInfo` instances representing struct fields.
    field_infos: FieldInfos<'a>,
}

impl<'a> StructInfo<'a> {
    /// Creates a new `StructInfo` instance from a `syn::DeriveInput`.
    ///
    /// # Arguments
    ///
    /// - `ast`: A `syn::DeriveInput` representing the input struct.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `StructInfo` instance if successful,
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
            if fields.named.is_empty() {
                return Err(syn::Error::new_spanned(fields, "No fields found"));
            }

            let mut settings = StructSettings::new().with_attrs(attrs)?;

            let field_infos = fields
                .named
                .iter()
                .map(|field| FieldInfo::new(field, &mut settings))
                .collect::<syn::Result<Vec<_>>>()?;

            let info = StructInfo {
                ident,
                vis,
                generics,
                builder_ident: format_ident!("{}{}", ident, settings.builder_suffix),
                data_ident: format_ident!("{}{}", ident, settings.data_suffix),
                _mandatory_indices: settings.mandatory_indices,
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
            .enumerate()
            .map(|(index, field)| FieldInfo::new(field, settings, index))
            .collect::<syn::Result<Vec<_>>>()
    }
  
    /// Retrieves the identifier of the struct.
    pub fn name(&self) -> &syn::Ident {
        self.ident
    }

    /// Retrieves the visibility of the struct.
    pub fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    /// Retrieves the generics of the struct.
    pub fn generics(&self) -> &syn::Generics {
        self.generics
    }

    /// Retrieves the identifier of the generated builder struct.
    pub fn builder_name(&self) -> &syn::Ident {
        &self.builder_ident
    }

    /// Retrieves the identifier of the generated data struct.
    pub fn data_name(&self) -> &syn::Ident {
        &self.data_ident
    }

    /// Retrieves a reference to the collection of `FieldInfo` instances representing struct fields.
    pub fn field_infos(&self) -> &FieldInfos {
        &self.field_infos
    }

    /// Retrieves a reference to the map of group names to their respective `GroupInfo`.
    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.groups
    }
}

/// Represents settings for struct generation.
#[derive(Debug)]
pub struct StructSettings {
    /// The suffix to be added to the generated builder struct name.
    builder_suffix: String,
    /// The suffix to be added to the generated data struct name.
    data_suffix: String,
    /// Default field settings.
    default_field_settings: FieldSettings,
    /// A map of group names to their respective `GroupInfo`.
    groups: HashMap<String, GroupInfo>,
    mandatory_indices: HashSet<usize>,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            builder_suffix: "Builder".to_string(),
            data_suffix: "Data".to_string(),
            default_field_settings: FieldSettings::new(),
            groups: HashMap::new(),
            mandatory_indices: HashSet::new(),
        }
    }
}

impl StructSettings {
    /// Creates a new `StructSettings` instance with default values.
    fn new() -> Self {
        Default::default()
    }

    pub fn add_mandatory_index(&mut self, index: usize) -> bool {
        self.mandatory_indices.insert(index)
    }

    pub fn group_by_name_mut(&mut self, group_name: &String) -> Option<&mut GroupInfo> {
        self.groups.get_mut(group_name)
    }

    /// Retrieves the default field settings.
    pub fn default_field_settings(&self) -> &FieldSettings {
        &self.default_field_settings
    }

    /// Updates struct settings based on provided attributes.
    ///
    /// # Arguments
    ///
    /// - `attrs`: A slice of `syn::Attribute` representing the attributes applied to the struct.
    ///
    /// # Returns
    ///
    /// A `syn::Result` indicating success or failure of attribute handling.
    pub fn with_attrs(mut self, attrs: &[syn::Attribute]) -> syn::Result<Self> {
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
            } else if ident == BUILDER {
                self.handle_builder_attribute(attr)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn handle_builder_attribute(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        let list = attr.meta.require_list()?;
        if list.tokens.is_empty() {
            return Ok(());
        }

        attr.parse_nested_meta(|meta| {
            if meta.path == ASSUME_MANDATORY {
                if meta.input.peek(Token![=]) {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(syn::LitBool { value, .. }),
                        ..
                    }) = expr
                    {
                        self.default_field_settings.mandatory = value;
                    }
                } else {
                    self.default_field_settings.mandatory = true;
                }
            }
            Ok(())
        })
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
                            .ok_or_else(|| syn::Error::new_spanned(func, "Can't parse group type"))
                    } else {
                        Err(syn::Error::new_spanned(func, "Can't find group type"))
                    }?;

                    let group_args = if let Some(syn::Expr::Lit(syn::ExprLit {
                        attrs: _,
                        lit: syn::Lit::Int(val),
                    })) = args.first()
                    {
                        val.base10_parse::<usize>()
                    } else {
                        Err(syn::Error::new_spanned(func, "Can't parse group args"))
                    }?;
                    match (&group_type.to_string()).into() {
                        EXACT => Ok(GroupType::Exact(group_args)),
                        AT_LEAST => Ok(GroupType::AtLeast(group_args)),
                        AT_MOST => Ok(GroupType::AtMost(group_args)),
                        SINGLE => Err(syn::Error::new_spanned(
                            args,
                            "`single` doesn't take any arguments",
                        )),
                        _ => Err(syn::Error::new_spanned(group_type, "Unknown group type")),
                    }
                }
                syn::Expr::Path(syn::ExprPath { path, .. }) => {
                    let group_type = path
                        .get_ident()
                        .ok_or_else(|| syn::Error::new_spanned(path, "Can't parse group type"))?;
                    match (&group_type.to_string()).into() {
                        EXACT | AT_LEAST | AT_MOST => Err(syn::Error::new_spanned(
                            &attr.meta,
                            "Missing arguments for group type",
                        )),
                        SINGLE => Ok(GroupType::Exact(1)),
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
