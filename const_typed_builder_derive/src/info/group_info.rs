use std::{collections::HashSet, hash::Hash};

use crate::symbol::{Symbol, AT_LEAST, AT_MOST, EXACT};

#[derive(Debug, Clone)]
pub struct GroupInfo {
    name: syn::Ident,
    associated_indices: HashSet<usize>,
    group_type: GroupType,
}

impl GroupInfo {
    pub fn new(name: syn::Ident, group_type: GroupType) -> Self {
        GroupInfo {
            name,
            associated_indices: HashSet::new(),
            group_type,
        }
    }

    pub fn name(&self) -> &syn::Ident {
        &self.name
    }

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

    // pub fn member_count(&self) -> usize {
    //     self.member_count
    // }

    // pub fn next_index(&mut self) -> usize {
    //     self.member_count += 1;
    //     self.member_count - 1
    // }

    // pub fn partial_const_ident(&self, index: usize) -> syn::Ident {
    //     format_ident!("{}_{}", &self.name.to_string().to_ascii_uppercase(), index)
    // }

    pub fn function_symbol(&self) -> Symbol {
        match self.group_type {
            GroupType::Exact(_) => EXACT,
            GroupType::AtLeast(_) => AT_LEAST,
            GroupType::AtMost(_) => AT_MOST,
        }
    }

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

#[derive(Debug, Clone)]
pub enum GroupType {
    Exact(usize),
    AtLeast(usize),
    AtMost(usize),
}
