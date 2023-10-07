use crate::{field_kind::FieldKind, symbol, util::is_option};
use proc_macro_error::{emit_error, emit_warning};
use std::collections::HashSet;

use super::Container;
use crate::info;

/// Represents settings for struct field generation.
#[derive(Debug, Clone)]
pub struct Field {
    kind: Option<FieldKind>,
    /// Indicates if the field should propagate values.
    propagate: bool,
    /// The groups this field belongs to.
    groups: HashSet<syn::Ident>,
}

// impl Default for Field {
//     fn default() -> Field {
//         Field {
//             index: 0,
//             ident: syn::Ident::new("", )
//             kind: None,
//             propagate: false,
//             groups: HashSet::new(),
//         }
//     }
// }

impl Field {
    /// Creates a new `FieldSettings` instance with default values.
    pub fn parse<'a>(
        ident: &'a syn::Ident,
        field: &'a syn::Field,
        container_parser: &mut Container,
        index: usize,
    ) -> info::Field<'a> {
        // let mut result = Self::default();
        let syn::Field { ty, attrs, .. } = field;

        let mut result = Self {
            kind: None,
            propagate: false,
            groups: HashSet::new(),
        };

        if result.kind.is_none() && !is_option(ty) {
            result.kind = Some(FieldKind::Mandatory); // If its not an option type it MUST always be mandatory
        }

        attrs
            .iter()
            .for_each(|attr: &syn::Attribute| result.handle_attribute(attr));

        match result.kind {
            Some(FieldKind::Grouped) => result.groups.iter().for_each(|group_name| {
                if let Some(group) = container_parser.group_by_name_mut(&group_name.to_string())
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
            Some(_) => {}
            None => if container_parser.assume_mandatory() {
                result.kind = Some(FieldKind::Mandatory)
            } else {
                result.kind = Some(FieldKind::Optional)
            },
        }

        info::Field::new(ident, ty, index, result.kind.unwrap(), result.propagate)
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
                symbol::SKIP => self.handle_skip(path_ident),
                symbol::MANDATORY => self.handle_mandatory(path_ident),
                symbol::OPTIONAL => self.handle_optional(path_ident),
                symbol::GROUP => self.handle_group(path_ident, &meta),
                symbol::PROPAGATE => self.propagate = true,
                _ => emit_error!(&attr.meta, "Unknown attribute"),
            }
            Ok(())
        })
        .unwrap_or_else(|err| {
            emit_error!(
                &attr.meta, "Unknown error";
                note = err
            )
        })
    }

    fn handle_skip(&mut self, ident: &syn::Ident) {
        match self.kind {
            None => self.kind = Some(FieldKind::Skipped),
            Some(FieldKind::Optional) => emit_error!(
                ident, "Can't define field as skipped as its already defined as optional";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Skipped) => {
                emit_warning!(ident, "Defined field as skipped multiple times")
            }
            Some(FieldKind::Mandatory) => emit_error!(
                ident, "Can't define field as skipped as its already defined as mandatory";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Grouped) => emit_error!(
                ident, "Can't define field as skipped when its also part of a group";
                hint = "Remove either types of attribute from this field"
            ),
        }
    }

    fn handle_mandatory(&mut self, ident: &syn::Ident) {
        match self.kind {
            None => self.kind = Some(FieldKind::Mandatory),
            Some(FieldKind::Optional) => emit_error!(
                ident, "Can't define field as mandatory as its already defined as optional";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Skipped) => emit_error!(
                ident, "Can't define field as mandatory as its already defined as skipped";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Mandatory) => {
                emit_warning!(ident, "Defined field as mandatory multiple times")
            }
            Some(FieldKind::Grouped) => emit_error!(
                ident, "Can't define field as mandatory when its also part of a group";
                hint = "Remove either types of attribute from this field"
            ),
        }
    }

    fn handle_optional(&mut self, ident: &syn::Ident) {
        match self.kind {
            None => self.kind = Some(FieldKind::Optional),
            Some(FieldKind::Optional) => {
                emit_warning!(ident, "Defined field as optional multiple times")
            }
            Some(FieldKind::Skipped) => emit_error!(
                ident, "Can't define field as optional as its already defined as skipped";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Mandatory) => emit_error!(
                ident, "Can't define field as optional as its already defined as mandatory";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Grouped) => emit_error!(
                ident, "Can't define field as optional when its also part of a group";
                hint = "Remove either types of attribute from this field"
            ),
        }
    }

    fn handle_group(&mut self, ident: &syn::Ident, meta: &syn::meta::ParseNestedMeta) {
        match self.kind {
            None => self.kind = Some(FieldKind::Grouped),
            Some(FieldKind::Optional) => emit_error!(
                ident, "Can't define field as part of a group as its already defined as optional";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Skipped) => emit_error!(
                ident, "Can't define field as as part of a group as its already defined as skipped";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Mandatory) => emit_error!(
                ident, "Can't define field as as part of a group as its already defined as mandatory";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Grouped) => {}
        }
        match self.extract_group_name(meta) {
            Ok(group_name) => {
                if self.groups.contains(&group_name) {
                    emit_error!(
                        group_name.span(), "Multiple adds to the same group";
                        help = self.groups.get(&group_name).unwrap().span() => "Remove this attribute"
                    );
                } else {
                    self.groups.insert(group_name);
                }
            }
            Err(err) => {
                emit_error!(
                    ident, "Group name not specified correctly";
                    help = "Try defining it like #[{}(foo)]", symbol::BUILDER;
                    note = err
                );
            }
        };
    }

    fn extract_group_name(&self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<syn::Ident> {
        match meta.value()?.parse()? {
            syn::Expr::Path(syn::ExprPath { path, .. }) => {
                path.require_ident().map(|ident| ident.clone())
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => Ok(syn::Ident::new(lit.value().as_str(), lit.span())),
            expr => Err(syn::Error::new_spanned(expr, "Unexpected expresion type")),
        }
    }
}
