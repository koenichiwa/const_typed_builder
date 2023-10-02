use super::field_info::{FieldInfo, FieldSettings};
use super::group_info::{GroupInfo, GroupType};
use crate::symbol::{
    ASSUME_MANDATORY, AT_LEAST, AT_MOST, BRUTE_FORCE, BUILDER, COMPILER, EXACT, GROUP, SINGLE,
    SOLVER,
};
use proc_macro_error::emit_error;
use quote::format_ident;
use std::collections::{BTreeSet, HashMap};
use syn::Token;

/// A type alias for a collection of `FieldInfo` instances.
type FieldInfos<'a> = Vec<FieldInfo<'a>>;

#[derive(Debug, Clone, Copy)]
pub enum SolveType {
    BruteForce,
    Compiler,
}
/// Represents the information about a struct used for code generation.
#[derive(Debug)]
pub struct StructInfo<'a> {
    /// The identifier of the struct.
    ident: &'a syn::Ident,
    /// The visibility of the struct.
    vis: &'a syn::Visibility,
    /// The generics of the struct.
    generics: &'a syn::Generics,
    /// The identifier of the generated builder struct.
    builder_ident: syn::Ident,
    /// The identifier of the generated data struct.
    data_ident: syn::Ident,
    _mandatory_indices: BTreeSet<usize>,
    /// A map of group names to their respective `GroupInfo`.
    groups: HashMap<String, GroupInfo>,
    /// A collection of `FieldInfo` instances representing struct fields.
    field_infos: FieldInfos<'a>,
    solve_type: SolveType,
}

impl<'a> StructInfo<'a> {
    /// Creates a new `StructInfo` instance from a `syn::DeriveInput`.
    ///
    /// # Arguments
    ///
    /// - `ast`: A `syn::DeriveInput` representing the input struct.
    ///
    /// # Returns
    ///
    /// A `syn::Result` containing the `StructInfo` instance if successful,
    pub fn new(ast: &'a syn::DeriveInput) -> Option<Self> {
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
                if fields.named.is_empty() {
                    emit_error!(fields, "No fields found");
                }

                let mut settings = StructSettings::new().with_attrs(attrs);

                let field_infos = fields
                    .named
                    .iter()
                    .enumerate()
                    .map(|(index, field)| FieldInfo::new(field, &mut settings, index))
                    .collect::<Option<Vec<_>>>()?;

                let info = StructInfo {
                    ident,
                    vis,
                    generics,
                    builder_ident: format_ident!("{}{}", ident, settings.builder_suffix),
                    data_ident: format_ident!("{}{}", ident, settings.data_suffix),
                    _mandatory_indices: settings.mandatory_indices,
                    groups: settings.groups,
                    field_infos,
                    solve_type: settings.solver_type,
                };
                Some(info)
            }
            _ => {
                emit_error!(ast, "Builder is only supported for named structs",);
                None
            }
        }
    }

    /// Retrieves the identifier of the struct.
    pub fn name(&self) -> &syn::Ident {
        self.ident
    }

    /// Retrieves the visibility of the struct.
    pub fn vis(&self) -> &syn::Visibility {
        self.vis
    }

    /// Retrieves the generics of the struct.
    pub fn generics(&self) -> &syn::Generics {
        self.generics
    }

    /// Retrieves the identifier of the generated builder struct.
    pub fn builder_name(&self) -> &syn::Ident {
        &self.builder_ident
    }

    /// Retrieves the identifier of the generated data struct.
    pub fn data_name(&self) -> &syn::Ident {
        &self.data_ident
    }

    /// Retrieves a reference to the collection of `FieldInfo` instances representing struct fields.
    pub fn field_infos(&self) -> &FieldInfos {
        &self.field_infos
    }

    /// Retrieves a reference to the map of group names to their respective `GroupInfo`.
    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.groups
    }

    pub fn solve_type(&self) -> SolveType {
        self.solve_type
    }
}

/// Represents settings for struct generation.
#[derive(Debug)]
pub struct StructSettings {
    /// The suffix to be added to the generated builder struct name.
    builder_suffix: String,
    /// The suffix to be added to the generated data struct name.
    data_suffix: String,
    /// Default field settings.
    default_field_settings: FieldSettings,
    /// A map of group names to their respective `GroupInfo`.
    groups: HashMap<String, GroupInfo>,
    mandatory_indices: BTreeSet<usize>,
    solver_type: SolveType,
}

impl Default for StructSettings {
    fn default() -> Self {
        StructSettings {
            builder_suffix: "Builder".to_string(),
            data_suffix: "Data".to_string(),
            default_field_settings: FieldSettings::new(),
            groups: HashMap::new(),
            mandatory_indices: BTreeSet::new(),
            solver_type: SolveType::BruteForce,
        }
    }
}

impl StructSettings {
    /// Creates a new `StructSettings` instance with default values.
    fn new() -> Self {
        StructSettings::default()
    }

    pub fn add_mandatory_index(&mut self, index: usize) -> bool {
        self.mandatory_indices.insert(index)
    }

    pub fn group_by_name_mut(&mut self, group_name: &String) -> Option<&mut GroupInfo> {
        self.groups.get_mut(group_name)
    }

    /// Retrieves the default field settings.
    pub fn default_field_settings(&self) -> &FieldSettings {
        &self.default_field_settings
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
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in handling the attribute. Errors are returned for invalid or conflicting attributes.
    fn handle_attribute(&mut self, attr: &syn::Attribute) {
        let path_ident = match attr.path().require_ident() {
            Ok(ident) => ident,
            Err(err) => {
                emit_error!(attr.path(), err);
                return;
            }
        };
        match (&path_ident.to_string()).into() {
            GROUP => self.handle_group_attribute(attr),
            BUILDER => self.handle_builder_attribute(attr),
            _ => emit_error!(&attr.meta, "Unknown attribute"),
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
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in handling the attribute. Errors are returned for invalid or conflicting attributes.
    fn handle_builder_attribute(&mut self, attr: &syn::Attribute) {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.tokens.is_empty() {
                    return;
                }
            }
            Err(err) => emit_error!(attr, err),
        };

        attr.parse_nested_meta(|meta| {
            let path_ident = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(&attr.meta, err);
                    return Ok(());
                }
            };

            match (&path_ident.to_string()).into() {
                ASSUME_MANDATORY => {
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Bool(syn::LitBool { value, .. }),
                            ..
                        }) = expr
                        {
                            self.default_field_settings.mandatory = value;
                        }
                    } else {
                        self.default_field_settings.mandatory = true;
                    }
                }
                SOLVER => {
                    if meta.input.peek(Token![=]) {
                        let expr: syn::Expr = meta.value()?.parse()?;
                        if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
                            if let Some(solve_type) = path.get_ident() {
                                match (&solve_type.to_string()).into() {
                                    BRUTE_FORCE => self.solver_type = SolveType::BruteForce,
                                    COMPILER => self.solver_type = SolveType::Compiler,
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
        .unwrap_or_else(|err| emit_error!(&attr.meta, err))
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
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure in handling the attribute. Errors are returned for invalid or conflicting attributes.
    fn handle_group_attribute(&mut self, attr: &syn::Attribute) {
        match attr.meta.require_list() {
            Ok(list) => {
                if list.tokens.is_empty() {
                    return;
                }
            }
            Err(err) => emit_error!(attr, err),
        };

        attr.parse_nested_meta(|meta| {
            let group_name = match meta.path.require_ident() {
                Ok(ident) => ident,
                Err(err) => {
                    emit_error!(&attr.meta, err);
                    return Ok(());
                }
            };

            let group_type = match meta.value()?.parse()? {
                syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                    let group_type = match func.as_ref() {
                        syn::Expr::Path(syn::ExprPath { path, .. }) => match path.require_ident() {
                            Ok(ident) => ident,
                            Err(err) => {
                                emit_error!(&attr.meta, err);
                                return Ok(());
                            }
                        },
                        _ => {
                            emit_error!(&attr.meta, "Can't find group type");
                            return Ok(());
                        }
                    };

                    match args.first() {
                        Some(syn::Expr::Lit(syn::ExprLit {
                            attrs: _,
                            lit: syn::Lit::Int(val),
                        })) => match val.base10_parse::<usize>() {
                            Ok(group_args) => match (&group_type.to_string()).into() {
                                EXACT => GroupType::Exact(group_args),
                                AT_LEAST => GroupType::AtLeast(group_args),
                                AT_MOST => GroupType::AtMost(group_args),
                                SINGLE => {
                                    emit_error!(args, "`single` doesn't take any arguments",);
                                    return Ok(());
                                }
                                _ => {
                                    emit_error!(group_type, "Unknown group type");
                                    return Ok(());
                                }
                            },
                            Err(err) => {
                                emit_error!(val, err);
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
                            emit_error!(path, err);
                            return Ok(());
                        }
                    };
                    match (&group_type.to_string()).into() {
                        EXACT | AT_LEAST | AT_MOST => {
                            emit_error!(&attr.meta, "Missing arguments for group type");
                            return Ok(());
                        }
                        SINGLE => GroupType::Exact(1),
                        _ => {
                            emit_error!(&attr.meta, "Can't parse group");
                            return Ok(());
                        }
                    }
                }
                _ => {
                    emit_error!(&attr.meta, "Can't find group type");
                    return Ok(());
                }
            };

            self.groups.insert(
                group_name.to_string(),
                GroupInfo::new(group_name.clone(), group_type),
            );
            Ok(())
        })
        .unwrap_or_else(|err| emit_error!(&attr.meta, err))
    }
}
