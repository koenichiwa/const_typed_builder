use proc_macro_error::{emit_error, emit_warning};

use crate::symbol::{Symbol, AT_LEAST, AT_MOST, EXACT};
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    hash::Hash,
};

pub type GroupCollection = HashMap<String, Group>;

/// Represents information about a group, including its name, member count, and group type.
#[derive(Debug, Clone)]
pub struct Group {
    name: syn::Ident,
    associated_indices: BTreeSet<usize>,
    group_type: GroupType,
}

impl Group {
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
        Group {
            name,
            associated_indices: BTreeSet::new(),
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
            GroupType::Exact(expected)
            | GroupType::AtLeast(expected)
            | GroupType::AtMost(expected) => expected,
        }
    }

    /// Associate a field index with this group
    pub fn associate(&mut self, index: usize) -> bool {
        self.associated_indices.insert(index)
    }

    /// Retrieves all associated indices
    pub fn indices(&self) -> &BTreeSet<usize> {
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

    pub fn is_valid_with(&self, indices: &[usize]) -> bool {
        let applicable_indices_count = self
            .associated_indices
            .intersection(&indices.iter().copied().collect())
            .count();
        match self.group_type {
            GroupType::Exact(count) => applicable_indices_count == count,
            GroupType::AtLeast(count) => applicable_indices_count >= count,
            GroupType::AtMost(count) => applicable_indices_count <= count,
        }
    }

    /// Check if the group is formed correctly. Will emit errors or warnings if invalid.
    pub fn check(&self) {
        let valid_range = 1..self.indices().len();
        if valid_range.is_empty() {
            emit_warning!(self.name, format!("There is not an valid expected count"))
        } else if !valid_range.contains(&self.expected_count()) {
            emit_warning!(
                self.name,
                format!("Expected count is outside of valid range {valid_range:#?}")
            );
        }
        match self.group_type() {
            GroupType::Exact(expected) => {
                match expected.cmp(&valid_range.start) {
                    Ordering::Less => emit_error!(
                        self.name,
                        "This group prevents all of the fields to be initialized";
                        hint = "Remove the group and use [builder(skip)] instead"
                    ),
                    Ordering::Equal | Ordering::Greater => {}
                }
                match expected.cmp(&valid_range.end) {
                    Ordering::Less => {}
                    Ordering::Equal => emit_warning!(
                        self.name,
                        "Group can only be satisfied if all fields are initialized";
                        hint = "Consider removing group and using [builder(mandatory)] instead"
                    ),
                    Ordering::Greater => emit_error!(
                        self.name,
                        "Group can never be satisfied";
                        note = format!("Expected amount of fields: exact {}, amount of available fields: {}", expected, valid_range.end)),
                }
            }
            GroupType::AtLeast(expected) => {
                match expected.cmp(&valid_range.start) {
                    Ordering::Less => emit_warning!(
                        self.name,
                        "Group has no effect";
                        hint = "Consider removing the group"
                    ),
                    Ordering::Equal | Ordering::Greater => {}
                }
                match expected.cmp(&valid_range.end) {
                    Ordering::Less => {}
                    Ordering::Equal => emit_warning!(
                        self.name,
                        "Group can only be satisfied if all fields are initialized";
                        hint = "Consider removing group and using [builder(mandatory)] instead"
                    ),
                    Ordering::Greater => emit_error!(
                        self.name,
                        "Group can never be satisfied";
                        note = format!("Expected amount of fields: at least {}, amount of available fields: {}", expected, valid_range.end);
                    ),
                }
            }
            GroupType::AtMost(expected) => {
                match expected.cmp(&valid_range.start) {
                    Ordering::Less => emit_error!(
                        self.name,
                        "This group prevents all of the fields to be initialized";
                        hint = "Remove the group and use [builder(skip)] instead";
                        note = format!("Expected amount of fields: at most {}, amount of available fields: {}", expected, valid_range.start)
                    ),
                    Ordering::Equal | Ordering::Greater => {}
                }
                match expected.cmp(&valid_range.end) {
                    Ordering::Less => {}
                    Ordering::Equal | Ordering::Greater => emit_warning!(
                        self.name,
                        "Group has no effect";
                        hint = "Consider removing the group"
                    ),
                }
            }
        }
    }
}

impl Eq for Group {}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Group {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.to_string().hash(state);
    }
}

/// Represents the type of a group, which can be one of three variants: `Exact`, `AtLeast`, or `AtMost`.
#[derive(Debug, Clone)]
pub enum GroupType {
    /// Represents a group with an exact member count.
    Exact(usize),
    /// Represents a group with at least a certain number of members.
    AtLeast(usize),
    /// Represents a group with at most a certain number of members.
    AtMost(usize),
}
