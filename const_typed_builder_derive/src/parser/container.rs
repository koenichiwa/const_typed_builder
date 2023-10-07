use std::collections::{BTreeSet, HashMap};

use proc_macro_error::{emit_error, emit_warning};
use syn::Token;

use crate::{
    info::{GroupInfo, GroupType},
    solver_kind::SolverKind,
    symbol,
};

/// Represents the parser for struct generation.
#[derive(Debug)]
pub struct Container {
    assume_mandatory: bool,
    /// A map of group names to their respective `GroupInfo`.
    groups: HashMap<String, GroupInfo>,
    /// The indices of the mandatory fields
    mandatory_indices: BTreeSet<usize>,
    /// The solver used to find all possible valid combinations for the groups
    solver_kind: SolverKind,
}

impl Default for Container {
    fn default() -> Self {
        Container {
            assume_mandatory: false,
            groups: HashMap::new(),
            mandatory_indices: BTreeSet::new(),
            solver_kind: SolverKind::BruteForce,
        }
    }
}

impl Container {
    pub fn assume_mandatory(&self) -> bool {
        self.assume_mandatory
    }

    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.groups
    }

    pub fn mandatory_indices(&self) -> &BTreeSet<usize> {
        &self.mandatory_indices
    }

    pub fn solver_kind(&self) -> SolverKind {
        self.solver_kind
    }

    /// Add a field index to the set of mandatory indices
    pub fn add_mandatory_index(&mut self, index: usize) -> bool {
        self.mandatory_indices.insert(index)
    }

    /// Get a GroupInfo by its identifier
    pub fn group_by_name_mut(&mut self, group_name: &String) -> Option<&mut GroupInfo> {
        self.groups.get_mut(group_name)
    }

    /// Updates struct settings based on provided attributes.
    ///
    /// # Arguments
    ///
    /// - `attrs`: A slice of `syn::Attribute` representing the attributes applied to the struct.
    ///
    /// # Returns
    ///
    /// A `syn::Result` indicating success or failure of attribute handling.
    pub fn with_attrs(mut self, attrs: &[syn::Attribute]) -> Self {
        attrs.iter().for_each(|attr| self.handle_attribute(attr));
        self
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
            symbol::GROUP => self.handle_group_attribute(attr),
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

    /// Handles the parsing and processing of group attributes applied to a struct.
    ///
    /// This method is responsible for interpreting the meaning of group attributes applied to the struct and
    /// updating the `StructSettings` accordingly. It supports the following group attributes:
    ///
    /// - `#[group(group_name = (exact(N)|at_least(N)|at_most(N)|single)]`:
    ///   Associates fields of the struct with a group named "group_name" and specifies the group's behavior.
    ///   The `group_name` should be a string identifier. The group can have one of the following behaviors:
    ///     - `exact(N)`: Exactly N fields in the group must be set during the builder construction.
    ///     - `at_least(N)`: At least N fields in the group must be set during the builder construction.
    ///     - `at_most(N)`: At most N fields in the group can be set during the builder construction.
    ///     - `single`: Only one field in the group can be set during the builder construction.
    ///
    /// # Arguments
    ///
    /// - `attr`: A reference to the `syn::Attribute` representing the group attribute applied to the struct.
    fn handle_group_attribute(&mut self, attr: &syn::Attribute) {
        attr.parse_nested_meta(|meta| {
            let group_name = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &meta.path , "Group name is not specified correctly";
                        help = "Try to define it like `#[{}(foo = {}(1))]`", symbol::GROUP, symbol::AT_LEAST;
                        note = err
                    );
                    return Ok(());
                }
            };

            let group_type = match meta.value()?.parse()? {
                syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                    let group_type = match func.as_ref() {
                        syn::Expr::Path(syn::ExprPath { path, .. }) => match path.require_ident() {
                            Ok(ident) => ident,
                            Err(err) => {
                                emit_error!(
                                    &meta.path , "Group type is not specified correctly";
                                    help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST;
                                    note = err
                                );
                                return Ok(());
                            }
                        },
                        _ => {
                            emit_error!(
                                &attr.meta, "No group type specified";
                                help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST
                            );
                            return Ok(());
                        }
                    };

                    match args.first() {
                        Some(syn::Expr::Lit(syn::ExprLit {
                            attrs: _,
                            lit: syn::Lit::Int(val),
                        })) => match val.base10_parse::<usize>() {
                            Ok(group_args) => match (&group_type.to_string()).into() {
                                symbol::EXACT => GroupType::Exact(group_args),
                                symbol::AT_LEAST => GroupType::AtLeast(group_args),
                                symbol::AT_MOST => GroupType::AtMost(group_args),
                                symbol::SINGLE => {
                                    emit_error!(
                                        args,
                                        "`{}` doesn't take any arguments", symbol::SINGLE;
                                        help = "`{}` is shorthand for {}(1)", symbol::SINGLE, symbol::EXACT
                                    );
                                    return Ok(());
                                }
                                _ => {
                                    emit_error!(
                                        group_type, "Unknown group type";
                                        help = "Known group types are {}, {} and {}", symbol::EXACT, symbol::AT_LEAST, symbol::AT_MOST
                                    );
                                    return Ok(());
                                }
                            },
                            Err(err) => {
                                emit_error!(
                                    val, "Couldn't parse group argument";
                                    note = err
                                );
                                return Ok(());
                            }
                        },

                        _ => {
                            emit_error!(func, "Can't parse group argument");
                            return Ok(());
                        }
                    }
                }
                syn::Expr::Path(syn::ExprPath { path, .. }) => {
                    let group_type = match path.require_ident() {
                        Ok(ident) => ident,
                        Err(err) => {
                            emit_error!(
                                &meta.path , "Group type is not specified correctly";
                                help = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST;
                                note = err
                            );
                            return Ok(());
                        }
                    };
                    match (&group_type.to_string()).into() {
                        symbol::EXACT | symbol::AT_LEAST | symbol::AT_MOST => {
                            emit_error!(
                                &attr.meta,
                                "Missing arguments for group type";
                                help = "Try `{}(1)`, or any other usize", &group_type
                            );
                            return Ok(());
                        }
                        symbol::SINGLE => GroupType::Exact(1),
                        _ => {
                            emit_error!(
                                group_type,
                                "Unknown group type";
                                help = "Known group types are {}, {} and {}", symbol::EXACT, symbol::AT_LEAST, symbol::AT_MOST
                            );
                            return Ok(());
                        }
                    }
                }
                _ => {
                    emit_error!(
                        &attr.meta, "No group type specified";
                        hint = "Try to define it like `#[group({} = {}(1))]`", group_name, symbol::AT_LEAST
                    );
                    return Ok(());
                }
            };

            self.groups.insert(
                group_name.to_string(),
                GroupInfo::new(group_name.clone(), group_type),
            );
            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(
            &attr.meta, "Unknown error";
            note = err
        ))
    }
}
