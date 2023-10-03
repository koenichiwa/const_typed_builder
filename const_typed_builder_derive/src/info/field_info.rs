use super::struct_info::StructSettings;
use crate::{
    symbol::{BUILDER, GROUP, MANDATORY, OPTIONAL, PROPAGATE, SKIP},
    CONST_IDENT_PREFIX,
};
use proc_macro2::Span;
use proc_macro_error::{emit_error, emit_warning};
use quote::format_ident;
use std::collections::HashSet;
use syn::{ExprPath, Token};

/// Represents the information about a struct field used for code generation.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldInfo<'a> {
    field: &'a syn::Field,
    ident: &'a syn::Ident,
    index: usize,
    propagate: bool,
    kind: FieldKind,
}

/// Represents the kind of a field, which can be Optional, Mandatory, or Grouped.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FieldKind {
    /// Indicates an optional field.
    Optional,
    /// Indicates a mandatory field.
    Mandatory,
    /// Indicates a field that is part of one or several groups.
    Grouped,
    /// Indicates a field that not included in the builder.
    Skipped,
}

impl<'a> FieldInfo<'a> {
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
    /// A `Result` containing the `FieldInfo` instance or an error if the field is unnamed.
    pub fn new(
        field: &'a syn::Field,
        struct_settings: &mut StructSettings,
        index: usize,
    ) -> Option<Self> {
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
                .with_attrs(attrs)
                .ok()?;

            let info = if settings.skipped {
                Self {
                    field,
                    ident,
                    index,
                    propagate: settings.propagate,
                    kind: FieldKind::Skipped,
                }
            } else if settings.mandatory {
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
                    if let Some(group) = struct_settings.group_by_name_mut(&group_name.to_string())
                    {
                        group.associate(index);
                    } else {
                        emit_error!(
                            group_name,
                            format!("No group called {group_name} is available");
                            hint = format!("You might want to add a #[{GROUP}(...)] attribute to the container")
                        );
                    }
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

            Some(info)
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
        util::is_option(&self.field.ty)
    }

    /// Retrieves the type of the field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }

    /// Retrieves the inner type of the field if it is wrapped in an Option
    pub fn inner_type(&self) -> Option<&syn::Type> {
        util::inner_type(&self.field.ty)
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
    pub skipped: bool,
    /// Indicates if the field is mandatory.
    pub mandatory: bool,
    /// Indicates if the field should propagate values.
    pub propagate: bool,
    /// The input name for the builder's setter method.
    pub input_name: syn::Ident,
    /// The groups this field belongs to.
    pub groups: HashSet<syn::Ident>,
}

impl Default for FieldSettings {
    fn default() -> FieldSettings {
        FieldSettings {
            skipped: false,
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
    fn with_attrs(mut self, attrs: &[syn::Attribute]) -> syn::Result<Self> {
        attrs.iter().for_each(|attr| self.handle_attribute(attr));
        // .collect::<Result<Vec<_>, _>>()?;
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
    fn with_ty(mut self, ty: &syn::Type) -> Self {
        if !self.mandatory && !util::is_option(ty) {
            self.mandatory = true;
        }
        self
    }

    /// Handles the parsing and processing of a builder attribute applied to a field.
    ///
    /// This method is responsible for interpreting the meaning of a builder attribute and updating the
    /// `FieldSettings` accordingly. It supports the following builder attributes:
    ///
    /// - `#[builder(mandatory)]`: Marks the field as mandatory, meaning it must be set during the builder
    ///   construction. If provided without an equals sign (e.g., `#[builder(mandatory)]`), it sets the field as mandatory.
    ///   If provided with an equals sign (e.g., `#[builder(mandatory = true)]`), it sets the mandatory flag based on the value.
    ///
    /// - `#[builder(optional)]`: Marks the field as optional, meaning it does not have to be set during
    ///   the builder construction. If provided without an equals sign (e.g., `#[builder(optional)]`), it sets the field as optional.
    ///   If provided with an equals sign (e.g., `#[builder(optional = true)]`), it sets the optional flag based on the value.
    ///
    /// - `#[builder(group = group_name)]`: Associates the field with a group named `group_name`. Fields in the same group
    ///   are treated as a unit, and at least one of them must be set during builder construction. If the field is marked as mandatory,
    ///   it cannot be part of a group. This attribute allows specifying the group name both as an identifier (e.g., `group = my_group`)
    ///   and as a string (e.g., `group = "my_group"`).
    ///
    /// - `#[builder(propagate)]`: Indicates that the field should propagate its value when the builder is constructed. If this attribute
    ///   is present, the field's value will be copied or moved to the constructed object when the builder is used to build the object.
    ///
    /// # Arguments
    ///
    /// - `attr`: A reference to the `syn::Attribute` representing the builder attribute applied to the field.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in handling the attribute. Errors are returned for invalid or conflicting attributes.
    fn handle_attribute(&mut self, attr: &syn::Attribute) {
        let attr_ident = match attr.path().require_ident() {
            Ok(ident) if ident == BUILDER => ident,
            Ok(ident) => {
                emit_error!(ident, format!("{ident} can't be used as a field attribute"));
                return;
            }
            Err(err) => {
                emit_error!(
                    attr.path(), "Can't parse attribute";
                    note = err
                );
                return;
            }
        };

        match attr.meta.require_list() {
            Ok(list) => {
                if list.tokens.is_empty() {
                    emit_warning!(list, "Empty atrribute list");
                }
            }
            Err(err) => emit_error!(
                attr, "Attribute expected contain a list of specifiers";
                help = "Try specifying it like #[{}(specifier)]", attr_ident;
                note = err
            ),
        }

        attr.parse_nested_meta(|meta| {
            let path_ident = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &attr.meta, "Specifier cannot be parsed";
                        help = "Try specifying it like #[{}(specifier)]", attr_ident;
                        note = err
                    );
                    return Ok(());
                }
            };

            match (&path_ident.to_string()).into() {
                SKIP => {
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Bool(syn::LitBool { value, .. }),
                            ..
                        }) = expr
                        {
                            self.skipped = value;
                        }
                    } else {
                        self.skipped = true;
                    }
                }
                MANDATORY => {
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
                OPTIONAL => {
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
                GROUP => {
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        let group_name = match expr {
                            syn::Expr::Path(ExprPath { path, .. }) => {
                                match path.require_ident() {
                                    Ok(ident) => ident,
                                    Err(err) => {
                                        emit_error!(
                                            path, "Group name not specified correctly";
                                            help = "Try defining it like #[{}(foo)]", BUILDER;
                                            note = err
                                        );
                                        return Ok(());
                                    }
                                }.clone()
                            }
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit),
                                ..
                            }) => {
                                syn::Ident::new(lit.value().as_str(), lit.span())
                            }
                            expr => {
                                emit_error!(expr, "Can't parse group name");
                                return Ok(());
                            },
                        };
                        if self.groups.contains(&group_name) {
                            emit_error!(
                                group_name.span(), "Multiple adds to the same group";
                                help = self.groups.get(&group_name).unwrap().span() => "Remove this attribute"
                            );
                        } else {
                            self.groups.insert(group_name);
                        }
                    }
                }
                PROPAGATE => {
                    self.propagate = true;
                }
                _ => {
                    emit_error!(&attr.meta, "Unknown attribute")
                }
            }

            if self.mandatory && !self.groups.is_empty() {
                emit_error!(
                    &meta.path,
                    format!("Can't use both {MANDATORY} and {GROUP} attributes");
                    hint = "Remove either types of attribute from this field"
                );
            }
            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(
            &attr.meta, "Unknown error";
            note = err
        ))
    }
}

mod util {
    pub fn is_option(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_some() {
                return false;
            }
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == syn::Ident::new("Option", segment.ident.span())
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn inner_type(ty: &syn::Type) -> Option<&syn::Type> {
        let path = if let syn::Type::Path(type_path) = ty {
            if type_path.qself.is_some() {
                return None;
            }
            &type_path.path
        } else {
            return None;
        };
        let segment = path.segments.last()?;
        let syn::PathArguments::AngleBracketed(generic_params) = &segment.arguments else {
            return None;
        };

        if let syn::GenericArgument::Type(inner) = generic_params.args.first()? {
            Some(inner)
        } else {
            None
        }
    }
}
