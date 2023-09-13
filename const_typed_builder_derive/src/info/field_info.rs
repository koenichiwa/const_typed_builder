use std::collections::{HashMap, HashSet};

use super::{group_info::GroupInfo, struct_info::StructSettings};
use proc_macro2::Span;
use syn::{ExprPath, Token};

use crate::{
    symbol::{BUILDER, GROUP, MANDATORY, PROPAGATE, OPTIONAL},
    util::{inner_type, is_option},
};

#[derive(Debug, PartialEq, Eq)]
pub enum FieldInfo<'a> {
    Optional(FieldInfoOptional<'a>),
    Mandatory(FieldInfoMandatory<'a>),
    Grouped(FieldInfoGrouped<'a>),
}

impl<'a> FieldInfo<'a> {
    pub fn new(field: &'a syn::Field, struct_settings: &mut StructSettings) -> syn::Result<Self> {
        if let syn::Field {
            attrs,
            ident: Some(ident),
            ty,
            vis: _,
            mutability: _,
            colon_token: _,
        } = field
        {
            let settings = struct_settings
                .default_field_settings()
                .clone()
                .with_ty(ty)
                .with_attrs(attrs)?;

            let info = if settings.mandatory {
                Self::Mandatory(FieldInfoMandatory::new(
                    field,
                    ident,
                    settings.propagate,
                    struct_settings.next_mandatory(),
                )?)
            } else if !settings.groups.is_empty() {
                let mut group_indices = HashMap::with_capacity(settings.groups.len());
                for group_name in settings.groups {
                    group_indices.insert(
                        struct_settings
                            .group_by_name(&group_name)
                            .ok_or(syn::Error::new_spanned(field, "Can't find group"))?
                            .clone(),
                        struct_settings
                            .next_group_index(&group_name)
                            .ok_or(syn::Error::new_spanned(field, "Can't find group"))?,
                    );
                }
                Self::Grouped(FieldInfoGrouped::new(
                    field,
                    ident,
                    settings.propagate,
                    group_indices,
                )?)
            } else {
                Self::Optional(FieldInfoOptional::new(field, ident, settings.propagate)?)
            };

            Ok(info)
        } else {
            Err(syn::Error::new_spanned(
                field,
                "Unnamed fields are not supported",
            ))
        }
    }

    pub fn ident(&self) -> &syn::Ident {
        match self {
            FieldInfo::Optional(field) => field.ident(),
            FieldInfo::Mandatory(field) => field.ident(),
            FieldInfo::Grouped(field) => field.ident(),
        }
    }

    pub fn propagate(&self) -> bool {
        match self {
            FieldInfo::Optional(field) => field.propagate(),
            FieldInfo::Mandatory(field) => field.propagate(),
            FieldInfo::Grouped(field) => field.propagate(),
        }
    }

    pub fn is_option_type(&self) -> bool {
        match self {
            FieldInfo::Optional(_) => true,
            FieldInfo::Mandatory(field) => field.is_option_type(),
            FieldInfo::Grouped(_) => true,
        }
    }

    pub fn inner_type(&self) -> Option<&syn::Type> {
        match self {
            FieldInfo::Optional(field) => Some(field.inner_type()),
            FieldInfo::Mandatory(field) => field.inner_type(),
            FieldInfo::Grouped(field) => Some(field.inner_type()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoOptional<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: &'a syn::Type,
    propagate: bool,
}

impl<'a> FieldInfoOptional<'a> {
    fn new(field: &'a syn::Field, ident: &'a syn::Ident, propagate: bool) -> syn::Result<Self> {
        Ok(Self {
            field,
            ident,
            inner_ty: inner_type(&field.ty)
                .ok_or(syn::Error::new_spanned(field, "Can't find inner type"))?,
            propagate,
        })
    }

    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    fn inner_type(&self) -> &syn::Type {
        self.inner_ty
    }

    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub fn propagate(&self) -> bool {
        self.propagate
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoMandatory<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: Option<&'a syn::Type>,
    propagate: bool,
    mandatory_index: usize,
}

impl<'a> FieldInfoMandatory<'a> {
    fn new(
        field: &'a syn::Field,
        ident: &'a syn::Ident,
        propagate: bool,
        mandatory_index: usize,
    ) -> syn::Result<Self> {
        Ok(Self {
            field,
            ident,
            inner_ty: inner_type(&field.ty),
            propagate,
            mandatory_index,
        })
    }

    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    pub fn inner_type(&self) -> Option<&syn::Type> {
        self.inner_ty
    }

    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub fn propagate(&self) -> bool {
        self.propagate
    }

    pub fn mandatory_index(&self) -> usize {
        self.mandatory_index
    }

    pub fn is_option_type(&self) -> bool {
        is_option(&self.field.ty)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoGrouped<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: &'a syn::Type,
    propagate: bool,
    group_indices: HashMap<GroupInfo, usize>,
}

impl<'a> FieldInfoGrouped<'a> {
    fn new(
        field: &'a syn::Field,
        ident: &'a syn::Ident,
        propagate: bool,
        group_indices: HashMap<GroupInfo, usize>,
    ) -> syn::Result<Self> {
        Ok(Self {
            field,
            ident,
            inner_ty: inner_type(&field.ty)
                .ok_or(syn::Error::new_spanned(field, "Can't find inner type"))?,
            propagate,
            group_indices,
        })
    }

    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    pub fn inner_type(&self) -> &syn::Type {
        self.inner_ty
    }

    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    pub fn propagate(&self) -> bool {
        self.propagate
    }

    pub fn group_indices(&self) -> &HashMap<GroupInfo, usize> {
        &self.group_indices
    }
}

#[derive(Debug, Clone)]
pub struct FieldSettings {
    pub mandatory: bool,
    pub propagate: bool,
    pub input_name: syn::Ident,
    pub groups: HashSet<String>,
}

impl Default for FieldSettings {
    fn default() -> FieldSettings {
        FieldSettings {
            mandatory: false,
            propagate: false,
            input_name: syn::Ident::new("input", Span::call_site()),
            groups: HashSet::new(),
        }
    }
}

impl FieldSettings {
    pub fn new() -> FieldSettings {
        Self::default()
    }

    pub fn with_attrs(mut self, attrs: &[syn::Attribute]) -> syn::Result<Self> {
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
        let list = attr.meta.require_list()?;
        if list.tokens.is_empty() {
            return Ok(());
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
            if meta.path == OPTIONAL {
                if meta.input.peek(Token![=]) {
                    let expr: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(syn::LitBool { value, .. }),
                        ..
                    }) = expr
                    {
                        self.mandatory = !value;
                    }
                } else {
                    self.mandatory = false;
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
                            .ok_or(syn::Error::new_spanned(path, "Can't parse group"))?;

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
            if meta.path == PROPAGATE {
                self.propagate = true;
            }
            Ok(())
        })
    }
}
