use super::field::Field;
use super::group_info::GroupInfo;
use crate::{parser, solver_kind::SolverKind};

use proc_macro_error::emit_error;
use quote::format_ident;
use std::collections::{BTreeSet, HashMap};

/// A type alias for a collection of `FieldInfo` instances.
type FieldCollection<'a> = Vec<Field<'a>>;

/// Represents the information about a struct used for code generation.
#[derive(Debug)]
pub struct Container<'a> {
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
    field_collection: FieldCollection<'a>,
    /// The solver used to find all possible valid combinations for the groups
    solve_type: SolverKind,
}

impl<'a> Container<'a> {
    /// Creates a new `StructInfo` instance from a `syn::DeriveInput`.
    ///
    /// # Arguments
    ///
    /// - `ast`: A `syn::DeriveInput` representing the input struct.
    ///
    /// # Returns
    ///
    /// An optional `StructInfo` instance if successful,
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

                let mut settings = parser::Container::default().with_attrs(attrs);

                let field_infos = fields
                    .named
                    .iter()
                    .enumerate()
                    .map(|(index, field)| Field::new(field, &mut settings, index))
                    .collect::<Option<Vec<_>>>()?;

                let info = Container {
                    ident,
                    vis,
                    generics,
                    builder_ident: format_ident!("{}{}", ident, "Builder"),
                    data_ident: format_ident!("{}{}", ident, "Data"),
                    _mandatory_indices: settings.mandatory_indices().clone(),
                    groups: settings.groups().clone(),
                    field_collection: field_infos,
                    solve_type: settings.solver_kind(),
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
    pub fn field_collection(&self) -> &FieldCollection {
        &self.field_collection
    }

    /// Retrieves a reference to the map of group names to their respective `GroupInfo`.
    pub fn groups(&self) -> &HashMap<String, GroupInfo> {
        &self.groups
    }

    /// Retrieves the solver type used to find all possible valid combinations for the groups
    pub fn solve_type(&self) -> SolverKind {
        self.solve_type
    }
}
