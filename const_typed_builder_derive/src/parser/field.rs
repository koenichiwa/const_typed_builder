use proc_macro_error::{emit_error, emit_warning};
use std::collections::HashSet;
use syn::Token;

use crate::{field_kind::FieldKind, symbol, util::is_option};

/// Represents settings for struct field generation.
#[derive(Debug, Clone)]
pub struct Field {
    kind: Option<FieldKind>,
    /// Indicates if the field should propagate values.
    propagate: bool,
    /// The groups this field belongs to.
    groups: HashSet<syn::Ident>,
}

impl Default for Field {
    fn default() -> Field {
        Field {
            kind: None,
            propagate: false,
            groups: HashSet::new(),
        }
    }
}

impl Field {
    pub fn kind(&self) -> Option<FieldKind> {
        self.kind
    }

    pub fn groups(&self) -> &HashSet<syn::Ident> {
        &self.groups
    }

    pub fn propagate(&self) -> bool {
        self.propagate
    }

    /// Creates a new `FieldSettings` instance with default values.
    pub fn handle_field(&self, field: &syn::Field, assume_mandatory: bool) -> Field {
        let syn::Field { ty, attrs, .. } = field;
        let mut result = self.clone();
        if result.kind == None && !is_option(ty) {
            result.kind = Some(FieldKind::Mandatory);
        }
        attrs
            .iter()
            .for_each(|attr: &syn::Attribute| result.handle_attribute(attr));
        if result.kind == None {
            if assume_mandatory {
                result.kind = Some(FieldKind::Mandatory)
            } else {
                result.kind = Some(FieldKind::Optional)
            }
        }
        result
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
    fn handle_attribute(&mut self, attr: &syn::Attribute) {
        let attr_ident = match attr.path().require_ident() {
            Ok(ident) if ident == symbol::BUILDER => ident,
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
                symbol::SKIP => {
                    match self.kind {
                        None => self.kind = Some(FieldKind::Skipped),
                        Some(FieldKind::Optional) => emit_error!(
                            path_ident, "Can't define field as skipped as its already defined as optional";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Skipped) => emit_warning!(path_ident, "Defined field as skipped multiple times"),
                        Some(FieldKind::Mandatory) => emit_error!(
                            path_ident, "Can't define field as skipped as its already defined as mandatory";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Grouped) => emit_error!(
                            path_ident, "Can't define field as skipped when its also part of a group";
                            hint = "Remove either types of attribute from this field"
                        ),
                    }
                }
                symbol::MANDATORY => {
                    match self.kind {
                        None => self.kind = Some(FieldKind::Mandatory),
                        Some(FieldKind::Optional) => emit_error!(
                            path_ident, "Can't define field as mandatory as its already defined as optional";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Skipped) => emit_error!(
                            path_ident, "Can't define field as mandatory as its already defined as skipped";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Mandatory) => emit_warning!(path_ident, "Defined field as mandatory multiple times"),
                        Some(FieldKind::Grouped) => emit_error!(
                            path_ident, "Can't define field as mandatory when its also part of a group";
                            hint = "Remove either types of attribute from this field"
                        ),
                    }
                }
                symbol::OPTIONAL => {
                    match self.kind {
                        None => self.kind = Some(FieldKind::Optional),
                        Some(FieldKind::Optional) => emit_warning!(path_ident, "Defined field as optional multiple times"),
                        Some(FieldKind::Skipped) => emit_error!(
                            path_ident, "Can't define field as optional as its already defined as skipped";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Mandatory) => emit_error!(
                            path_ident, "Can't define field as optional as its already defined as mandatory";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Grouped) => emit_error!(
                            path_ident, "Can't define field as optional when its also part of a group";
                            hint = "Remove either types of attribute from this field"
                        ),
                    }
                }
                symbol::GROUP => {
                    match self.kind {
                        None => self.kind = Some(FieldKind::Grouped),
                        Some(FieldKind::Optional) => emit_error!(
                            path_ident, "Can't define field as part of a group as its already defined as optional";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Skipped) => emit_error!(
                            path_ident, "Can't define field as as part of a group as its already defined as skipped";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Mandatory) => emit_error!(
                            path_ident, "Can't define field as as part of a group as its already defined as mandatory";
                            hint = "Remove either types of attribute from this field"
                        ),
                        Some(FieldKind::Grouped) => {},
                    }
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        let group_name = match expr {
                            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                                match path.require_ident() {
                                    Ok(ident) => ident,
                                    Err(err) => {
                                        emit_error!(
                                            path, "Group name not specified correctly";
                                            help = "Try defining it like #[{}(foo)]", symbol::BUILDER;
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
                symbol::PROPAGATE => {
                    self.propagate = true;
                }
                _ => {
                    emit_error!(&attr.meta, "Unknown attribute")
                }
            }
            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(
            &attr.meta, "Unknown error";
            note = err
        ))
    }
}
