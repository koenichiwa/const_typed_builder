use std::{
    borrow::BorrowMut,
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use proc_macro2::Span;
use quote::format_ident;
use syn::{Attribute, ExprPath, Token};

use crate::{
    context::Context,
    group::Group,
    struct_info::StructSettings,
    symbol::{BUILDER, GROUP, MANDATORY},
    util::{inner_type, is_option},
    MANDATORY_PREFIX,
};

#[derive(Debug)]
pub struct FieldInfo<'a> {
    field: &'a syn::Field,
    attrs: &'a Vec<syn::Attribute>,
    vis: &'a syn::Visibility,
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    index: usize,
    mandatory_index: Option<usize>,
    group_indices: HashMap<Group, usize>,
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
        context: &mut Context,
        index: usize,
        field: &'a syn::Field,
        struct_settings: &StructSettings,
    ) -> Option<Self> {
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
                .with_attrs(context, attrs)?;

            let info = FieldInfo {
                field,
                attrs,
                vis,
                ident,
                ty,
                index,
                mandatory_index: None,
                group_indices: HashMap::new(), // Set by struct_info
                settings,
            };

            Some(info)
        } else {
            context.error_spanned_by(field, "Cannot parse field");
            None
        }
    }

    pub fn group_names(&self) -> &HashSet<String> {
        &self.settings.groups
    }

    pub fn set_mandatory_index(&mut self, mandatory_index: usize) {
        self.mandatory_index = Some(mandatory_index);
    }

    pub fn set_group_index(&mut self, group: Group, index: usize) {
        self.group_indices.insert(group, index);
    }

    pub fn get_group_index(&self, group: &Group) -> Option<usize> {
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

    pub fn type_kind(&self, context: &mut Context) -> Option<TypeKind> {
        let type_kind = if is_option(self.ty) {
            let inner_ty = inner_type(self.ty)
                .ok_or_else(|| {
                    context.error_spanned_by(self.ty, "Cannot read inner type");
                })
                .ok()?;

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
        Some(type_kind)
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
        context: &mut Context,
        attrs: &Vec<syn::Attribute>,
    ) -> Option<Self> {
        if let Err(err) = attrs
            .iter()
            .map(|attr| self.handle_attribute(attr))
            .collect::<Result<Vec<_>, _>>()
        {
            context.syn_error(err);
            None
        } else {
            Some(self)
        }
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
