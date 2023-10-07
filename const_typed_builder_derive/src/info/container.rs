use super::field::FieldCollection;
use super::group::GroupCollection;
use crate::solver_kind::SolverKind;

use quote::format_ident;

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
    /// A map of group names to their respective `GroupInfo`.
    groups: GroupCollection,
    /// A collection of `FieldInfo` instances representing struct fields.
    field_collection: FieldCollection<'a>,
    /// The solver used to find all possible valid combinations for the groups
    solver_kind: SolverKind,
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
    pub fn new(
        vis: &'a syn::Visibility,
        generics: &'a syn::Generics,
        ident: &'a syn::Ident,
        group_collection: GroupCollection,
        field_collection: FieldCollection<'a>,
        solver_kind: SolverKind,
    ) -> Self {
        Container {
            ident,
            vis,
            generics,
            builder_ident: format_ident!("{}{}", ident, "Builder"),
            data_ident: format_ident!("{}{}", ident, "Data"),
            groups: group_collection,
            field_collection,
            solver_kind,
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
    pub fn groups(&self) -> &GroupCollection {
        &self.groups
    }

    /// Retrieves the solver type used to find all possible valid combinations for the groups
    pub fn solve_type(&self) -> SolverKind {
        self.solver_kind
    }
}
