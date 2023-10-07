use crate::{
    field_kind::FieldKind,
    parser, symbol,
    util::{inner_type, is_option},
    CONST_IDENT_PREFIX,
};
use proc_macro_error::emit_error;
use quote::format_ident;

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub struct Field<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    index: usize,
    propagate: bool,
    kind: FieldKind,
}

// /// Represents the kind of a field, which can be Optional, Mandatory, or Grouped.
// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub enum FieldKind {
//     /// Indicates an optional field.
//     Optional,
//     /// Indicates a mandatory field.
//     Mandatory,
//     /// Indicates a field that is part of one or several groups.
//     Grouped,
//     /// Indicates a field that not included in the builder.
//     Skipped,
// }

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
        field: &'a syn::Field,
        struct_parser: &mut parser::Container,
        index: usize,
    ) -> Option<Self> {
        if let syn::Field {
            ident: Some(ident), ..
        } = field
        {
            let field_parser =
                parser::Field::default().handle_field(field, struct_parser.assume_mandatory());
            let kind = field_parser
                .kind()
                .expect("Kind should always be set after parsing");

            match kind {
                FieldKind::Optional | FieldKind::Skipped => {},
                FieldKind::Mandatory => assert!(struct_parser.add_mandatory_index(index)),
                FieldKind::Grouped => field_parser.groups().iter().for_each(|group_name| {
                    if let Some(group) = struct_parser.group_by_name_mut(&group_name.to_string())
                    {
                        group.associate(index);
                    } else {
                        emit_error!(
                            group_name,
                            "No group called {} is available", group_name;
                            hint = "You might want to add a #[{}(...)] attribute to the container", symbol::GROUP
                        );
                    }
                }),
            };

            Some(Self {
                field,
                ident,
                index,
                propagate: field_parser.propagate(),
                kind,
            })
        } else {
            emit_error!(field, "Unnamed fields are not supported");
            None
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
        is_option(&self.field.ty)
    }

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    /// Retrieves the inner type of the field if it is wrapped in an Option
    pub fn inner_type(&self) -> Option<&syn::Type> {
        inner_type(&self.field.ty)
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
