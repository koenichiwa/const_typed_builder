use std::collections::HashMap;

use proc_macro_error::{emit_error, emit_warning};
use syn::Token;

use crate::{info, solver_kind::SolverKind, symbol};

use super::{group::Group, Field};

/// Represents the parser for struct generation.
#[derive(Debug)]
pub struct Container {
    assume_mandatory: bool,
    /// A map of group names to their respective `GroupInfo`.
    groups: info::GroupCollection,
    /// The solver used to find all possible valid combinations for the groups
    solver_kind: SolverKind,
}

impl Container {
    /// Updates struct settings based on provided attributes.
    ///
    /// # Arguments
    ///
    /// - `attrs`: A slice of `syn::Attribute` representing the attributes applied to the struct.
    ///
    /// # Returns
    ///
    /// A `syn::Result` indicating success or failure of attribute handling.
    pub fn parse(ast: &syn::DeriveInput) -> Option<info::Container> {
        match &ast {
            syn::DeriveInput {
                attrs,
                vis,
                ident,
                generics,
                data:
                    syn::Data::Struct(syn::DataStruct {
                        fields: syn::Fields::Named(fields),
                        ..
                    }),
            } => {
                let mut result = Container {
                    assume_mandatory: false,
                    groups: HashMap::new(),
                    solver_kind: SolverKind::BruteForce,
                };

                attrs.iter().for_each(|attr| result.handle_attribute(attr));

                let fields = fields
                    .named
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        assert!(field.ident.is_some());
                        Field::parse(
                            field
                                .ident
                                .as_ref()
                                .expect("FieldsNamed should have a named field"),
                            field,
                            &mut result,
                            index,
                        )
                    })
                    .collect::<Vec<_>>();

                Some(info::Container::new(
                    vis,
                    generics,
                    ident,
                    result.groups,
                    fields,
                    result.solver_kind,
                ))
            }
            _ => {
                emit_error!(ast, "Builder is only supported for named structs",);
                None
            }
        }
    }

    pub fn assume_mandatory(&self) -> bool {
        self.assume_mandatory
    }

    /// Get a GroupInfo by its identifier
    pub fn group_by_name_mut(&mut self, group_name: &String) -> Option<&mut info::Group> {
        self.groups.get_mut(group_name)
    }

    /// Handles the parsing and processing of attributes applied to a struct.
    ///
    /// See the specific functions [`handle_builder_attribute`] and [`handle_group_attribute`] for more information.
    ///
    /// /// # Arguments
    ///
    /// - `attr`: A reference to the `syn::Attribute` representing the builder attribute applied to the struct.
    fn handle_attribute(&mut self, attr: &syn::Attribute) {
        let attr_ident = match attr.path().require_ident() {
            Ok(ident) => ident,
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
        match (&attr_ident.to_string()).into() {
            symbol::GROUP => {
                if let Some(group) = Group::parse(attr) {
                    self.groups.insert(group.name().to_string(), group);
                }
            }
            symbol::BUILDER => self.handle_builder_attribute(attr),
            _ => emit_error!(&attr, "Unknown attribute"),
        }
    }

    /// Handles the parsing and processing of builder attributes applied to a struct.
    ///
    /// This method is responsible for interpreting the meaning of builder attributes applied to the struct and
    /// updating the `StructSettings` accordingly. It supports the following builder attributes:
    ///
    /// - `#[builder(assume_mandatory)]`: Indicates that all fields in the struct should be assumed as mandatory.
    ///   If provided without an equals sign (e.g., `#[builder(assume_mandatory)]`), it sets the `mandatory` flag for fields to true.
    ///   If provided with an equals sign (e.g., `#[builder(assume_mandatory = true)]`), it sets the `mandatory` flag for fields based on the value.
    ///
    /// - `#[builder(solver = `solve_type`)]`: Specifies the solver type to be used for building the struct. The `solve_type` should be one of
    ///   the predefined solver types, such as `brute_force` or `compiler`. If provided with an equals sign (e.g., `#[builder(solver = brute_force)]`),
    ///   it sets the `solver_type` accordingly.
    ///
    /// # Arguments
    ///
    /// - `attr`: A reference to the `syn::Attribute` representing the builder attribute applied to the struct.
    fn handle_builder_attribute(&mut self, attr: &syn::Attribute) {
        attr.parse_nested_meta(|meta| {
            let path_ident = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &attr.meta, "Specifier cannot be parsed";
                        help = "Try specifying it like #[{}(specifier)]", symbol::BUILDER;
                        note = err
                    );
                    return Ok(());
                }
            };

            match (&path_ident.to_string()).into() {
                symbol::ASSUME_MANDATORY => {
                    self.assume_mandatory = true;
                }
                symbol::SOLVER => {
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
                            if let Some(solve_type) = path.get_ident() {
                                match (&solve_type.to_string()).into() {
                                    symbol::BRUTE_FORCE => {
                                        self.solver_kind = SolverKind::BruteForce
                                    }
                                    symbol::COMPILER => self.solver_kind = SolverKind::Compiler,
                                    _ => emit_error!(&path, "Unknown solver type"),
                                }
                            } else {
                                emit_error!(meta.path, "Can't parse solver specification");
                            }
                        } else {
                            emit_error!(meta.path, "Can't parse solver specification");
                        }
                    } else {
                        emit_error!(meta.path, "Solver type needs to be specified");
                    }
                }
                _ => {
                    emit_error!(meta.path, "Unknown attribute");
                }
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
}
