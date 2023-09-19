use std::{collections::BTreeSet, hash::Hash};

/// Represents information about a group, including its name, member count, and group type.
#[derive(Debug, Clone)]
pub struct GroupInfo {
    name: syn::Ident,
    associated_indices: BTreeSet<usize>,
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
            associated_indices: BTreeSet::new(),
            group_type,
        }
    }

    pub fn associate(&mut self, index: usize) -> bool {
        self.associated_indices.insert(index)
    }

    pub fn indices(&self) -> &BTreeSet<usize> {
        &self.associated_indices
    }

    pub fn is_valid_with(&self, indices: &[usize]) -> bool {
        let applicable_indices_count = self
            .associated_indices
            .intersection(&BTreeSet::from_iter(indices.iter().map(|idx| *idx)))
            .count();
        match self.group_type {
            GroupType::Exact(count) => applicable_indices_count == count,
            GroupType::AtLeast(count) => applicable_indices_count >= count,
            GroupType::AtMost(count) => applicable_indices_count <= count,
        }
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
