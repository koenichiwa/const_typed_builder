use super::field::FieldCollection;
use super::group::GroupCollection;
use convert_case::{Case, Casing};
use quote::format_ident;

#[derive(Debug, Clone, Copy)]
pub enum SolverKind {
    BruteForce,
    Compiler,
}

/// Represents the information about a struct used for code generation.
#[derive(Debug)]
pub struct Container<'a> {
    /// The identifier of the struct.
    ident: &'a syn::Ident,
    /// The visibility of the struct.
    vis: &'a syn::Visibility,
    /// The generics of the struct.
    generics: &'a syn::Generics,
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
            groups: group_collection,
            field_collection,
            solver_kind,
        }
    }

    /// Retrieves the identifier of the struct.
    pub fn ident(&self) -> &syn::Ident {
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
    pub fn builder_ident(&self) -> syn::Ident {
        format_ident!("{}{}", self.ident, "Builder")
    }

    /// Retrieves the identifier of the generated data struct.
    pub fn data_ident(&self) -> syn::Ident {
        format_ident!("{}{}", self.ident, "Data")
    }

    /// Retrieves a reference to the collection of `FieldInfo` instances representing struct fields.
    pub fn field_collection(&self) -> &FieldCollection {
        &self.field_collection
    }

    /// Retrieves a reference to the map of group names to their respective `GroupInfo`.
    pub fn group_collection(&self) -> &GroupCollection {
        &self.groups
    }

    /// Retrieves the solver type used to find all possible valid combinations for the groups
    pub fn solver_kind(&self) -> SolverKind {
        self.solver_kind
    }

    pub fn data_field_ident(&self) -> syn::Ident {
        format_ident!("__{}", self.data_ident().to_string().to_case(Case::Snake))
    }

    pub fn mod_ident(&self) -> syn::Ident {
        format_ident!("{}", self.builder_ident().to_string().to_case(Case::Snake))
    }

    pub fn generate_module(&self) -> bool {
        false
    }
}
