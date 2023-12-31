use crate::{
    info::{Field, FieldKind, GroupCollection, SetterKind},
    symbol::Symbol,
    util::is_option,
};
use proc_macro_error::{emit_error, emit_warning};
use std::str::FromStr;

/// Represents settings for struct field generation.
#[derive(Debug)]
pub struct FieldParser<'parser> {
    kind: Option<FieldKind>,
    setter_kind: Option<SetterKind>,
    index: usize,
    assume_mandatory: bool,
    assume_into: bool,
    group_collection: &'parser mut GroupCollection,
}

impl<'parser> FieldParser<'parser> {
    pub fn new(
        index: usize,
        assume_mandatory: bool,
        assume_into: bool,
        group_collection: &'parser mut GroupCollection,
    ) -> Self {
        Self {
            kind: None,
            setter_kind: None,
            index,
            assume_mandatory,
            assume_into,
            group_collection,
        }
    }

    pub fn parse<'ast>(mut self, ident: &'ast syn::Ident, field: &'ast syn::Field) -> Field<'ast> {
        let syn::Field { ty, attrs, .. } = field;

        if !is_option(ty) {
            self.kind = Some(FieldKind::Mandatory); // If its not an option type it MUST always be mandatory
        }

        attrs
            .iter()
            .for_each(|attr: &syn::Attribute| self.handle_attribute(attr));

        if self.kind.is_none() {
            self.kind = if self.assume_mandatory {
                Some(FieldKind::Mandatory)
            } else {
                Some(FieldKind::Optional)
            };
        }

        if self.setter_kind.is_none() {
            self.setter_kind = if self.assume_into {
                Some(SetterKind::Into)
            } else {
                Some(SetterKind::Standard)
            };
        }

        Field::new(
            ident,
            ty,
            self.index,
            self.kind.unwrap(),
            self.setter_kind.unwrap(),
        )
    }

    /// Handles the parsing and processing of a builder attribute applied to a field.
    ///
    /// This method is responsible for interpreting the meaning of a builder attribute and updating the
    /// `FieldSettings` accordingly. It supports the following builder attributes:
    ///
    /// - `#[builder(mandatory)]`: Marks the field as mandatory, meaning it must be set during the builder
    ///   construction.
    ///
    /// - `#[builder(optional)]`: Marks the field as optional, meaning it does not have to be set during
    ///   the builder construction.
    ///
    /// - `#[builder(skipped)]`: Marks the field as skipped, meaning it can't be set during
    ///   the builder construction.
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
            Ok(ident) if Symbol::from_str(&ident.to_string()) == Ok(Symbol::Builder) => ident,
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

            match Symbol::from_str(&path_ident.to_string()) {
                Ok(symbol) => match symbol {
                    Symbol::Skip => self.handle_attribute_skip(path_ident),
                    Symbol::Mandatory => self.handle_attribute_mandatory(path_ident),
                    Symbol::Optional => self.handle_attribute_optional(path_ident),
                    Symbol::Group => self.handle_attribute_group(&meta),
                    Symbol::Propagate => self.handle_setter_kind(SetterKind::Propagate, path_ident),
                    Symbol::AsRef => self.handle_setter_kind(SetterKind::AsRef, path_ident),
                    Symbol::AsMut => self.handle_setter_kind(SetterKind::AsMut, path_ident),
                    Symbol::Into => self.handle_setter_kind(SetterKind::Into, path_ident),
                    Symbol::Standard => self.handle_setter_kind(SetterKind::Standard, path_ident),
                    symbol => {
                        emit_error!(&attr.meta, format!("Specifier {symbol} can't be used here"))
                    }
                },
                Err(err) => emit_error!(
                    &attr.meta, "Unknown attribute";
                    note = err
                ),
            }
            Ok(())
        })
        .unwrap_or_else(|err| {
            emit_error!(
                &attr.meta, "An error occured while parsing this attribute";
                note = err
            )
        })
    }

    fn handle_setter_kind(&mut self, kind: SetterKind, ident: &syn::Ident) {
        if self.setter_kind.is_some() {
            emit_error!(ident, "Setter type defined multiple times");
        } else {
            self.setter_kind = Some(kind);
        }
    }

    fn handle_attribute_skip(&mut self, ident: &syn::Ident) {
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

    fn handle_attribute_mandatory(&mut self, ident: &syn::Ident) {
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

    fn handle_attribute_optional(&mut self, ident: &syn::Ident) {
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

    fn handle_attribute_group(&mut self, meta: &syn::meta::ParseNestedMeta) {
        match self.kind {
            None => self.kind = Some(FieldKind::Grouped),
            Some(FieldKind::Optional) => emit_error!(
                meta.path, "Can't define field as part of a group as its already defined as optional";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Skipped) => emit_error!(
                meta.path, "Can't define field as as part of a group as its already defined as skipped";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Mandatory) => emit_error!(
                meta.path, "Can't define field as as part of a group as its already defined as mandatory";
                hint = "Remove either types of attribute from this field"
            ),
            Some(FieldKind::Grouped) => {}
        }
        match self.extract_group_name(meta) {
            Ok(group_name) => {
                if let Some(group) = self.group_collection.get_mut(&group_name.to_string()) {
                    if group.indices().contains(&self.index) {
                        emit_warning!(
                            group_name.span(), "Multiple adds to the same group";
                            help = "Remove this attribute"
                        )
                    } else {
                        group.associate(self.index);
                    }
                }
            }
            Err(err) => {
                emit_error!(
                    meta.path, "Group name not specified correctly";
                    help = "Try defining it like #[{}(foo)]", Symbol::Builder;
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
