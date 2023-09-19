use std::collections::HashSet;

use super::struct_info::StructSettings;
use proc_macro2::Span;
use quote::format_ident;
use syn::{ExprPath, Token};

use crate::{
    symbol::{BUILDER, GROUP, MANDATORY, OPTIONAL, PROPAGATE},
    util::{inner_type, is_option},
    CONST_IDENT_PREFIX,
};

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfo<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    index: usize,
    propagate: bool,
    kind: FieldKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldKind {
    Optional,
    Mandatory,
    Grouped,
}

impl<'a> FieldInfo<'a> {
    pub fn new(
        field: &'a syn::Field,
        struct_settings: &mut StructSettings,
        index: usize,
    ) -> syn::Result<Self> {
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
                struct_settings.add_mandatory_index(index); // TODO: Check bool
                Self {
                    field,
                    ident,
                    index,
                    propagate: settings.propagate,
                    kind: FieldKind::Mandatory,
                }
            } else if !settings.groups.is_empty() {
                for group_name in settings.groups {
                    struct_settings
                        .group_by_name_mut(&group_name.to_string())
                        .ok_or(syn::Error::new_spanned(group_name, "Can't find group"))?
                        .associate(index);
                }

                Self {
                    field,
                    ident,
                    index,
                    propagate: settings.propagate,
                    kind: FieldKind::Grouped,
                }
            } else {
                Self {
                    field,
                    ident,
                    index,
                    propagate: settings.propagate,
                    kind: FieldKind::Optional,
                }
            };

            Ok(info)
        } else {
            Err(syn::Error::new_spanned(
                field,
                "Unnamed fields are not supported",
            ))
        }
    }

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    /// Checks if the field's attributes indicate propagation.
    pub fn propagate(&self) -> bool {
        self.propagate
    }

    pub fn is_option_type(&self) -> bool {
        is_option(&self.field.ty)
    }

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    pub fn inner_type(&self) -> Option<&syn::Type> {
        inner_type(&self.field.ty)
    }

    pub fn kind(&self) -> &FieldKind {
        &self.kind
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn const_ident(&self) -> syn::Ident {
        format_ident!("{}{}", CONST_IDENT_PREFIX, self.index)
    }

    /// Retrieves the input type for the builder's setter method.
    pub fn setter_input_type(&self) -> &syn::Type {
        match self.kind() {
            FieldKind::Optional => self.ty(),
            FieldKind::Mandatory if self.is_option_type() => self.inner_type().expect(
                "Couldn't read inner type of option, even though it's seen as an Option type",
            ),
            FieldKind::Mandatory => self.ty(),
            FieldKind::Grouped => self
                .inner_type()
                .expect("Couldn't read inner type of option, even though it's marked as grouped"),
        }
    }
}

impl<'a> PartialOrd for FieldInfo<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<'a> Ord for FieldInfo<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

/// Represents settings for struct field generation.
#[derive(Debug, Clone)]
pub struct FieldSettings {
    /// Indicates if the field is mandatory.
    pub mandatory: bool,
    /// Indicates if the field should propagate values.
    pub propagate: bool,
    /// The input name for the builder's setter method.
    pub input_name: syn::Ident,
    pub groups: HashSet<syn::Ident>,
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
    /// Creates a new `FieldSettings` instance with default values.
    pub fn new() -> FieldSettings {
        Self::default()
    }

    /// Updates field settings based on provided attributes.
    ///
    /// # Arguments
    ///
    /// - `attrs`: A slice of `syn::Attribute` representing the attributes applied to the field.
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

    /// Updates field settings based on the field's type.
    ///
    /// # Arguments
    ///
    /// - `ty`: A reference to the `syn::Type` representing the field's type.
    ///
    /// # Returns
    ///
    /// The updated `FieldSettings` instance.
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

                        if !self.groups.insert(group_name.clone()) {
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
                        if !self
                            .groups
                            .insert(syn::Ident::new(lit.value().as_str(), lit.span()))
                        {
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
