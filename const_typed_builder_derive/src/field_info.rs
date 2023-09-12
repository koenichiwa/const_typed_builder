use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::{ExprPath, Token};

use crate::{
    group_info::GroupInfo,
    struct_info::StructSettings,
    symbol::{BUILDER, GROUP, MANDATORY},
    util::{inner_type, is_option},
};

enum FieldInfoKind<'a> {
    Optional { ident: &'a syn::Ident, ty: &'a syn::Type, inner_ty: &'a syn::Type, settings: FieldSettings },
    Mandatory { ident: &'a syn::Ident, ty: &'a syn::Type, inner_ty: Option<&'a syn::Type>, settings: FieldSettings, mandatory_index: usize},
    Grouped { ident: &'a syn::Ident, ty: &'a syn::Type, inner_ty: &'a syn::Type, settings: FieldSettings, group_indices: HashMap<GroupInfo, usize>,}
}

impl <'a> FieldInfoKind <'a> {
    pub fn new(
        field: &'a syn::Field,
        struct_settings: &StructSettings,
    ) -> syn::Result<Self> {
        if let syn::Field {
            attrs,
            vis,
            mutability,
            ident: Some(ident),
            ty,
            ..
        } = field
        {
            let settings = struct_settings
                .default_field_settings()
                .clone()
                .with_ty(ty)
                .with_attrs(attrs)?;

            let info = if settings.mandatory {
                Self::Mandatory { ident, ty, inner_ty: inner_type(ty), settings, mandatory_index: 0 }
            } else if !settings.groups.is_empty() {
                Self::Grouped { ident, ty, inner_ty: inner_type(ty).ok_or(syn::Error::new_spanned(field, "Can't find inner type"))?, settings, group_indices: HashMap::new() }
            } else {
                Self::Optional { ident , ty, inner_ty: inner_type(ty).ok_or(syn::Error::new_spanned(field, "Can't find inner type"))?, settings }
            };

            Ok(info)
        } else {
            Err(syn::Error::new_spanned(field, "Unnamed fields are not supported"))
        }
    }
}

#[derive(Debug)]
pub struct FieldInfo<'a> {
    field: &'a syn::Field,
    attrs: &'a Vec<syn::Attribute>,
    vis: &'a syn::Visibility,
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    mandatory_index: Option<usize>,
    group_indices: HashMap<GroupInfo, usize>,
    settings: FieldSettings,
}

#[derive(Debug, Clone)]
pub struct FieldSettings {
    pub mandatory: bool,
    pub input_name: syn::Ident,
    pub groups: HashSet<String>,
}

impl<'a> FieldInfo<'a> {
    pub fn new(
        field: &'a syn::Field,
        struct_settings: &StructSettings,
    ) -> syn::Result<Self> {
        if let syn::Field {
            attrs,
            vis,
            mutability,
            ident: Some(ident),
            ty,
            ..
        } = field
        {
            let settings = struct_settings
                .default_field_settings()
                .clone()
                .with_ty(ty)
                .with_attrs(attrs)?;

            let info = FieldInfo {
                field,
                attrs,
                vis,
                ident,
                ty,
                mandatory_index: None,
                group_indices: HashMap::new(), // Set by struct_info
                settings,
            };

            Ok(info)
        } else {
            Err(syn::Error::new_spanned(field, "Unnamed fields are not supported"))
        }
    }

    pub fn group_names(&self) -> &HashSet<String> {
        &self.settings.groups
    }

    pub fn set_mandatory_index(&mut self, mandatory_index: usize) {
        self.mandatory_index = Some(mandatory_index);
    }

    pub fn set_group_index(&mut self, group: GroupInfo, index: usize) {
        self.group_indices.insert(group, index);
    }

    pub fn get_group_index(&self, group: &GroupInfo) -> Option<usize> {
        self.group_indices.get(group).copied()
    }

    pub fn name(&self) -> &syn::Ident {
        self.ident
    }

    pub fn mandatory(&self) -> bool {
        self.settings.mandatory
    }

    pub fn input_name(&self) -> &syn::Ident {
        &self.settings.input_name
    }

    pub fn type_kind(&self) -> syn::Result<TypeKind> {
        let type_kind = if is_option(self.ty) {
            let inner_ty = inner_type(self.ty)
                .ok_or_else(|| syn::Error::new_spanned(self.ty, "Cannot read inner type"))?;

            if !self.group_names().is_empty() {
                TypeKind::GroupOption {
                    ty: self.ty,
                    inner_ty,
                }
            } else if self.settings.mandatory {
                TypeKind::MandatoryOption {
                    ty: self.ty,
                    inner_ty,
                }
            } else {
                TypeKind::Optional {
                    ty: self.ty,
                    inner_ty,
                }
            }
        } else {
            if self.settings.mandatory {
                TypeKind::Mandatory { ty: self.ty }
            } else {
                unreachable!("Non-optional types are always mandatory")
            }
        };
        Ok(type_kind)
    }

    pub fn mandatory_index(&self) -> Option<usize> {
        self.mandatory_index
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

    pub fn with_ty(mut self, ty: &syn::Type) -> Self {
        if !self.mandatory && !is_option(ty) {
            self.mandatory = true;
        }
        self
    }

    fn handle_attribute(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if let Some(ident) = attr.path().get_ident() {
            if ident != BUILDER {
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
                        self.mandatory = value;
                    }
                } else {
                    self.mandatory = true;
                }
            }
            if meta.path == GROUP {
                if self.mandatory {
                    return Err(syn::Error::new_spanned(
                        &meta.path,
                        "Only optionals in group",
                    ));
                }
                if meta.input.peek(Token![=]) {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Path(ExprPath { path, .. }) = &expr {
                        let group_name = path
                            .get_ident()
                            .ok_or(syn::Error::new_spanned(&path, "Can't parse group"))?;

                        if !self.groups.insert(group_name.to_string()) {
                            return Err(syn::Error::new_spanned(
                                &expr,
                                "Multiple adds to the same group",
                            ));
                        }
                    }
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) = &expr
                    {
                        if !self.groups.insert(lit.value()) {
                            return Err(syn::Error::new_spanned(
                                &expr,
                                "Multiple adds to the same group",
                            ));
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

#[derive(Debug)]
pub enum TypeKind<'a> {
    Mandatory {
        ty: &'a syn::Type,
    },
    MandatoryOption {
        ty: &'a syn::Type,
        inner_ty: &'a syn::Type,
    },
    Optional {
        ty: &'a syn::Type,
        inner_ty: &'a syn::Type,
    },
    GroupOption {
        ty: &'a syn::Type,
        inner_ty: &'a syn::Type,
    },
}
