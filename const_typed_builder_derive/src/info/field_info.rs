use std::collections::{HashMap, HashSet};

use super::{group_info::GroupInfo, struct_info::StructSettings};
use proc_macro2::Span;
use syn::{ExprPath, Token};

use crate::{
    symbol::{BUILDER, GROUP, MANDATORY, OPTIONAL, PROPAGATE},
    util::{inner_type, is_option},
};

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub enum FieldInfo<'a> {
    /// Represents an optional field.
    Optional(FieldInfoOptional<'a>),
    /// Represents a mandatory field.
    Mandatory(FieldInfoMandatory<'a>),
    /// Represents a grouped field.
    Grouped(FieldInfoGrouped<'a>),
}

impl<'a> FieldInfo<'a> {
    /// Creates a new `FieldInfo` instance from a `syn::Field` and `StructSettings`.
    ///
    /// # Arguments
    ///
    /// - `field`: A `syn::Field` representing the input field.
    /// - `struct_settings`: A mutable reference to `StructSettings`.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `FieldInfo` instance if successful, or an error if parsing fails.
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

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        match self {
            FieldInfo::Optional(field) => field.ident(),
            FieldInfo::Mandatory(field) => field.ident(),
            FieldInfo::Grouped(field) => field.ident(),
        }
    }

    /// Retrieves whether the field's attributes indicate builder propagation.
    pub fn propagate(&self) -> bool {
        match self {
            FieldInfo::Optional(field) => field.propagate(),
            FieldInfo::Mandatory(field) => field.propagate(),
            FieldInfo::Grouped(field) => field.propagate(),
        }
    }

    /// Checks if the field's type is an `Option`.
    pub fn is_option_type(&self) -> bool {
        match self {
            FieldInfo::Optional(_) => true,
            FieldInfo::Mandatory(field) => field.is_option_type(),
            FieldInfo::Grouped(_) => true,
        }
    }

    /// Retrieves the inner type of the field if it is an `Option`.
    pub fn inner_type(&self) -> Option<&syn::Type> {
        match self {
            FieldInfo::Optional(field) => Some(field.inner_type()),
            FieldInfo::Mandatory(field) => field.inner_type(),
            FieldInfo::Grouped(field) => Some(field.inner_type()),
        }
    }

    /// Retrieves the input type for the builder's setter method.
    pub fn setter_input_type(&self) -> &syn::Type {
        match self {
            FieldInfo::Optional(field) => field.ty(),
            FieldInfo::Mandatory(field) if field.is_option_type() => field
                .inner_type()
                .expect("Couldn't read inner type of option, even though it's marked as optional"),
            FieldInfo::Mandatory(field) => field.ty(),
            FieldInfo::Grouped(field) => field.inner_type(),
        }
    }
}

/// Represents information about an optional field.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoOptional<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: &'a syn::Type,
    propagate: bool,
}

impl<'a> FieldInfoOptional<'a> {
    /// Creates a new `FieldInfoOptional` instance from a `syn::Field`.
    ///
    /// # Arguments
    ///
    /// - `field`: A `syn::Field` representing the input field.
    /// - `ident`: A reference to the identifier of the field.
    /// - `propagate`: A boolean indicating whether the field should propagate values.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `FieldInfoOptional` instance if successful, or an error if parsing fails.

    fn new(field: &'a syn::Field, ident: &'a syn::Ident, propagate: bool) -> syn::Result<Self> {
        Ok(Self {
            field,
            ident,
            inner_ty: inner_type(&field.ty)
                .ok_or(syn::Error::new_spanned(field, "Can't find inner type"))?,
            propagate,
        })
    }

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    /// Retrieves the inner type of the field.
    fn inner_type(&self) -> &syn::Type {
        self.inner_ty
    }

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    /// Checks if the field's attributes indicate builder propagation.
    pub fn propagate(&self) -> bool {
        self.propagate
    }
}

/// Represents information about a mandatory field.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoMandatory<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: Option<&'a syn::Type>,
    propagate: bool,
    mandatory_index: usize,
}

impl<'a> FieldInfoMandatory<'a> {
    /// Creates a new `FieldInfoMandatory` instance from a `syn::Field`.
    ///
    /// # Arguments
    ///
    /// - `field`: A `syn::Field` representing the input field.
    /// - `ident`: A reference to the identifier of the field.
    /// - `propagate`: A boolean indicating whether the field should propagate values.
    /// - `mandatory_index`: The index of the mandatory field.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `FieldInfoMandatory` instance if successful, or an error if parsing fails.
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

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    /// Retrieves the inner type of the field.
    pub fn inner_type(&self) -> Option<&syn::Type> {
        self.inner_ty
    }

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    /// Checks if the field's attributes indicate propagation.
    pub fn propagate(&self) -> bool {
        self.propagate
    }

    /// Retrieves the index of the mandatory field.
    pub fn mandatory_index(&self) -> usize {
        self.mandatory_index
    }

    /// Checks if the field's type is an `Option`.
    pub fn is_option_type(&self) -> bool {
        is_option(&self.field.ty)
    }
}

/// Represents information about a grouped field.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfoGrouped<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    inner_ty: &'a syn::Type,
    propagate: bool,
    group_indices: HashMap<GroupInfo, usize>,
}

impl<'a> FieldInfoGrouped<'a> {
    /// Creates a new `FieldInfoGrouped` instance from a `syn::Field`.
    ///
    /// # Arguments
    ///
    /// - `field`: A `syn::Field` representing the input field.
    /// - `ident`: A reference to the identifier of the field.
    /// - `propagate`: A boolean indicating whether the field should propagate values.
    /// - `group_indices`: A map of `GroupInfo` to group indices.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `FieldInfoGrouped` instance if successful, or an error if parsing fails.
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

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    /// Retrieves the inner type of the field.
    pub fn inner_type(&self) -> &syn::Type {
        self.inner_ty
    }

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    /// Checks if the field's attributes indicate propagation.
    pub fn propagate(&self) -> bool {
        self.propagate
    }

    /// Retrieves the group indices associated with the field.
    pub fn group_indices(&self) -> &HashMap<GroupInfo, usize> {
        &self.group_indices
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
    /// A set of group names associated with the field.
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
