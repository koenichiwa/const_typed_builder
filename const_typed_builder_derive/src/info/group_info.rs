use std::{collections::HashSet, hash::Hash};

use crate::symbol::{Symbol, AT_LEAST, AT_MOST, EXACT};

/// Represents information about a group, including its name, member count, and group type.
#[derive(Debug, Clone)]
pub struct GroupInfo {
    name: syn::Ident,
    associated_indices: HashSet<usize>,
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
            associated_indices: HashSet::new(),
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

    pub fn associate(&mut self, index: usize) -> bool {
        self.associated_indices.insert(index)
    }

    pub fn indices(&self) -> &HashSet<usize> {
        &self.associated_indices
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
