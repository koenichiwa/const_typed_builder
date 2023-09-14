use std::hash::Hash;

use quote::format_ident;

use crate::symbol::{Symbol, AT_LEAST, AT_MOST, EXACT};

/// Represents information about a group, including its name, member count, and group type.
#[derive(Debug, Clone)]
pub struct GroupInfo {
    name: syn::Ident,
    member_count: usize,
    group_type: GroupType,
}

impl GroupInfo {
    /// Creates a new `GroupInfo` instance.
    ///
    /// # Arguments
    ///
    /// - `name`: The identifier representing the group's name.
    /// - `group_type`: The type of the group.
    ///
    /// # Returns
    ///
    /// A `GroupInfo` instance with the provided name and group type.
    pub fn new(name: syn::Ident, group_type: GroupType) -> Self {
        GroupInfo {
            name,
            member_count: 0,
            group_type,
        }
    }

    /// Retrieves the name of the group.
    pub fn name(&self) -> &syn::Ident {
        &self.name
    }

    /// Retrieves the expected member count based on the group type.
    pub fn expected_count(&self) -> usize {
        match self.group_type {
            GroupType::Exact(expected) => expected,
            GroupType::AtLeast(expected) => expected,
            GroupType::AtMost(expected) => expected,
        }
    }

    /// Retrieves the current member count of the group.
    pub fn member_count(&self) -> usize {
        self.member_count
    }

    /// Increments the member count and returns the next available index.
    pub fn next_index(&mut self) -> usize {
        self.member_count += 1;
        self.member_count - 1
    }

    /// Generates a partial constant identifier for the group at the given index.
    ///
    /// # Arguments
    ///
    /// - `index`: The index for which to generate the partial constant identifier.
    ///
    /// # Returns
    ///
    /// A `syn::Ident` representing the partial constant identifier.
    pub fn partial_const_ident(&self, index: usize) -> syn::Ident {
        format_ident!("{}_{}", &self.name.to_string().to_ascii_uppercase(), index)
    }

    /// Retrieves the function symbol associated with the group type.
    pub fn function_symbol(&self) -> Symbol {
        match self.group_type {
            GroupType::Exact(_) => EXACT,
            GroupType::AtLeast(_) => AT_LEAST,
            GroupType::AtMost(_) => AT_MOST,
        }
    }

    /// Retrieves the group type.
    pub fn group_type(&self) -> &GroupType {
        &self.group_type
    }
}

impl Eq for GroupInfo {}

impl PartialEq for GroupInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for GroupInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_string().hash(state);
    }
}

/// Represents the type of a group, which can be one of three variants: Exact, AtLeast, or AtMost.
#[derive(Debug, Clone)]
pub enum GroupType {
    /// Represents a group with an exact member count.
    Exact(usize),
    /// Represents a group with at least a certain number of members.
    AtLeast(usize),
    /// Represents a group with at most a certain number of members.
    AtMost(usize),
}
