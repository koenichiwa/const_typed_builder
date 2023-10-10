use crate::util::{inner_type, is_option};
use quote::format_ident;
use std::ops::Deref;

/// A type alias for a collection of `FieldInfo` instances.
pub type FieldCollection<'a> = Vec<Field<'a>>;

const CONST_IDENT_PREFIX: &str = "__BUILDER_CONST_";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FieldKind {
    Optional,
    Skipped,
    Mandatory,
    Grouped,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SetterKind {
    Standard,
    Propagate,
    Into,
    AsMut,
    AsRef,
}

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> {
    ty: &'a syn::Type,
    ident: &'a syn::Ident,
    index: usize,
    kind: FieldKind,
    setter_kind: SetterKind,
}

impl<'a> Field<'a> {
    /// Creates a new `FieldInfo` instance for a struct field.
    ///
    /// # Arguments
    ///
    /// - `field`: A reference to the `syn::Field` representing the field.
    /// - `struct_settings`: A mutable reference to `StructSettings` for the struct containing this field.
    /// - `index`: The index of the field within the struct.
    ///
    /// # Returns
    ///
    /// An otpional `FieldInfo` instance if successful.
    pub fn new(
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
        index: usize,
        kind: FieldKind,
        setter_kind: SetterKind,
    ) -> Self {
        Self {
            ident,
            index,
            ty,
            kind,
            setter_kind,
        }
    }

    /// Retrieves the identifier of the field.
    pub fn ident(&self) -> &syn::Ident {
        self.ident
    }

    /// Checks if the field's type is an Option.
    pub fn is_option_type(&self) -> bool {
        is_option(self.ty)
    }

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        self.ty
    }

    /// Retrieves the inner type of the field if it is wrapped in an Option
    pub fn inner_type(&self) -> Option<&syn::Type> {
        inner_type(self.ty)
    }

    /// Retrieves the kind of the field, which can be Optional, Mandatory, Skipped or Grouped.
    pub fn kind(&self) -> FieldKind {
        self.kind
    }

    /// Retrieves the index of the field within the struct.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Generates a constant identifier based on the field's index.
    pub fn const_ident(&self) -> syn::Ident {
        format_ident!("{}{}", CONST_IDENT_PREFIX, self.index)
    }

    pub fn setter_kind(&self) -> SetterKind {
        self.setter_kind
    }
}

impl<'a> PartialOrd for Field<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<'a> Ord for Field<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}
pub enum TrackedFieldKind {
    Mandatory,
    Grouped,
}
pub struct TrackedField<'a> {
    field: &'a Field<'a>,
    kind: TrackedFieldKind,
}

impl<'a> TrackedField<'a> {
    /// Creates a [`TrackedField`] if the input [`Field`] is Mandatory or Grouped.
    pub fn new(field: &'a Field) -> Option<Self> {
        match field.kind() {
            FieldKind::Optional | FieldKind::Skipped => None,
            FieldKind::Mandatory => Some(Self {
                field,
                kind: TrackedFieldKind::Mandatory,
            }),
            FieldKind::Grouped => Some(Self {
                field,
                kind: TrackedFieldKind::Grouped,
            }),
        }
    }
    /// Retrieves the kind of the field, which can be Mandatory, or Grouped.
    pub fn kind(&self) -> &TrackedFieldKind {
        &self.kind
    }
}

impl<'a> Deref for TrackedField<'a> {
    type Target = Field<'a>;

    fn deref(&self) -> &Self::Target {
        self.field
    }
}
