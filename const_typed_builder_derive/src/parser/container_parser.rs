use super::{FieldParser, GroupParser};
use crate::{
    info::{Container, Field, FieldCollection, GroupCollection, SolverKind},
    symbol::Symbol,
};
use proc_macro_error::{emit_call_site_error, emit_error, emit_warning};
use std::str::FromStr;

/// Represents the parser for struct generation.
#[derive(Debug)]
pub struct ContainerParser {
    assume_mandatory: bool,
    assume_into: bool,
    /// A map of group names to their respective `GroupInfo`.
    groups: GroupCollection,
    /// The solver used to find all possible valid combinations for the groups
    solver_kind: SolverKind,
}

impl ContainerParser {
    pub fn new() -> Self {
        Self::default()
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
    pub fn parse(mut self, ast: &syn::DeriveInput) -> Option<Container> {
        let syn::DeriveInput {
            attrs,
            vis,
            ident,
            generics,
            data,
        } = ast;

        attrs.iter().for_each(|attr| self.handle_attribute(attr));

        let fields = self.handle_data(data)?;

        Some(Container::new(
            vis,
            generics,
            ident,
            self.groups,
            fields,
            self.solver_kind,
        ))
    }

    /// Handles the parsing and processing of attributes applied to a struct.
    ///
    /// See the specific functions [`handle_attribute_builder`] and [`handle_attribute_group`] for more information.
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
        match Symbol::from_str(&attr_ident.to_string()) {
            Ok(symbol) => match symbol {
                Symbol::Group => {
                    emit_warning!(&attr_ident, "The use of group as a top level attribute is being deprecated, use groups instead");
                    GroupParser::new(&mut self.groups).parse(attr)
                }
                Symbol::Groups => GroupParser::new(&mut self.groups).parse(attr),
                Symbol::Builder => self.handle_attribute_builder(attr),
                symbol => {
                    emit_error!(
                        &attr.meta,
                        format!("Attribute {symbol} can't be used at container level")
                    )
                }
            },
            Err(err) => emit_error!(
                &attr_ident, "Unknown symbol";
                note = err
            ),
        }
    }

    /// Handles the parsing and processing of builder attributes applied to a struct.
    ///
    /// This method is responsible for interpreting the meaning of builder attributes applied to the struct and
    /// updating the `StructSettings` accordingly. It supports the following builder attributes:
    ///
    /// - `#[builder(assume_mandatory)]`: Indicates that all fields in the struct should be assumed as mandatory.
    ///
    /// - `#[builder(solver = `solve_type`)]`: Specifies the solver type to be used for building the struct. The `solve_type` should be one of
    ///   the predefined solver types, such as `brute_force` or `compiler`. If provided with an equals sign (e.g., `#[builder(solver = brute_force)]`),
    ///   it sets the `solver_type` accordingly.
    ///
    /// # Arguments
    ///
    /// - `attr`: A reference to the `syn::Attribute` representing the builder attribute applied to the struct.
    fn handle_attribute_builder(&mut self, attr: &syn::Attribute) {
        attr.parse_nested_meta(|meta| {
            let path_ident = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(
                        &attr.meta, "Specifier cannot be parsed";
                        help = "Try specifying it like #[{}(specifier)]", Symbol::Builder;
                        note = err
                    );
                    return Ok(());
                }
            };
            match Symbol::from_str(&path_ident.to_string()) {
                Ok(symbol) => match symbol {
                    Symbol::Solver => {
                        let syn::ExprPath { path, .. } = meta.value()?.parse()?;
                        match Symbol::from_str(&path.require_ident()?.to_string()) {
                            Ok(solver) => match solver {
                                Symbol::BruteForce => self.solver_kind = SolverKind::BruteForce,
                                Symbol::Compiler => self.solver_kind = SolverKind::Compiler,
                                solver => {
                                    emit_error!(&path, format!("{solver} is not a solver type"))
                                }
                            },
                            Err(err) => emit_error!(
                                &path, "Unknown symbol";
                                note = err
                            ),
                        }
                    }
                    Symbol::AssumeMandatory => self.assume_mandatory = true,
                    Symbol::Into => self.assume_into = true,
                    symbol => {
                        emit_error!(
                            &attr.meta,
                            format!("Specifier {symbol} can't be used at container level")
                        )
                    }
                },
                Err(err) => emit_error!(
                    &attr.meta, "Unknown symbol";
                    note = err
                ),
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

    fn handle_data<'a>(&mut self, data: &'a syn::Data) -> Option<FieldCollection<'a>> {
        match data {
            syn::Data::Struct(syn::DataStruct { fields, .. }) => self.handle_fields(fields),
            syn::Data::Enum(syn::DataEnum { variants, .. }) => {
                let _ = variants
                    .iter()
                    .map(|variant| self.handle_fields(&variant.fields));
                emit_call_site_error!("Builder does not *yet* support enums",);
                None
            }
            syn::Data::Union(_) => {
                emit_call_site_error!("Builder does not support unions",);
                None
            }
        }
    }

    fn handle_fields<'a>(&mut self, fields: &'a syn::Fields) -> Option<Vec<Field<'a>>> {
        match fields {
            syn::Fields::Named(fields) => Some(self.handle_named_fields(fields)),
            syn::Fields::Unnamed(fields) => {
                emit_error!(fields, "Builder does not support unnamed fields");
                None
            }
            syn::Fields::Unit => Some(Vec::new()),
        }
    }

    fn handle_named_fields<'a>(&mut self, fields: &'a syn::FieldsNamed) -> Vec<Field<'a>> {
        fields
            .named
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let ident = field
                    .ident
                    .as_ref()
                    .expect("FieldsNamed should have an ident");
                FieldParser::new(
                    index,
                    self.assume_mandatory,
                    self.assume_into,
                    &mut self.groups,
                )
                .parse(ident, field)
            })
            .collect::<Vec<_>>()
    }
}

impl Default for ContainerParser {
    fn default() -> Self {
        ContainerParser {
            assume_mandatory: false,
            assume_into: false,
            groups: GroupCollection::new(),
            solver_kind: SolverKind::BruteForce,
        }
    }
}
