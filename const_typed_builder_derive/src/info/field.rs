use crate::{
    field_kind::FieldKind,
    util::{inner_type, is_option},
    CONST_IDENT_PREFIX,
};
use quote::format_ident;

/// A type alias for a collection of `FieldInfo` instances.
pub type FieldCollection<'a> = Vec<Field<'a>>;

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> {
    ty: &'a syn::Type,
    ident: &'a syn::Ident,
    index: usize,
    propagate: bool,
    kind: FieldKind,
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
        propagate: bool,
    ) -> Self {
        Self {
            ident,
            index,
            ty,
            propagate,
            kind,
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

    /// Retrieves the kind of the field, which can be Optional, Mandatory, or Grouped.
    pub fn kind(&self) -> &FieldKind {
        &self.kind
    }

    /// Retrieves the index of the field within the struct.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Generates a constant identifier based on the field's index.
    pub fn const_ident(&self) -> syn::Ident {
        format_ident!("{}{}", CONST_IDENT_PREFIX, self.index)
    }

    /// Retrieves the input type for the builder's setter method.
    pub fn setter_input_type(&self) -> Option<&syn::Type> {
        match self.kind() {
            FieldKind::Optional => Some(self.ty()),
            FieldKind::Mandatory if self.is_option_type() => Some(self.inner_type().expect(
                "Couldn't read inner type of option, even though it's seen as an Option type",
            )),
            FieldKind::Mandatory => Some(self.ty()),
            FieldKind::Grouped => {
                Some(self.inner_type().expect(
                    "Couldn't read inner type of option, even though it's marked as grouped",
                ))
            }
            FieldKind::Skipped => None,
        }
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