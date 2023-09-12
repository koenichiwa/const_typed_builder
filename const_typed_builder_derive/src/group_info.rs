use std::hash::Hash;

use quote::format_ident;

#[derive(Debug, Clone)]
pub struct GroupInfo {
    name: String,
    member_count: usize,
    group_type: GroupType,
}

impl GroupInfo {
    pub fn new(name: String, group_type: GroupType) -> Self {
        GroupInfo {
            name,
            member_count: 0,
            group_type,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn expected_count(&self) -> usize {
        match self.group_type {
            GroupType::Exact(expected) => expected,
            GroupType::AtLeast(expected) => expected,
            GroupType::AtMost(expected) => expected,
        }
    }

    pub fn member_count(&self) -> usize {
        self.member_count
    }

    pub fn incr_member_count(&mut self) {
        self.member_count += 1;
    }

    pub fn const_ident(&self) -> syn::Ident {
        format_ident!("{}", &self.name.to_ascii_uppercase())
    }

    pub fn partial_const_ident(&self, index: usize) -> syn::Ident {
        format_ident!("{}_{}", &self.name.to_ascii_uppercase(), index)
    }

    pub fn function_ident(&self) -> syn::Ident {
        match self.group_type {
            GroupType::Exact(_) => format_ident!("{}", "exact"),
            GroupType::AtLeast(_) => format_ident!("{}", "at_least"),
            GroupType::AtMost(_) => format_ident!("{}", "at_most"),
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
        self.name.hash(state);
    }
}

#[derive(Debug, Clone)]
pub enum GroupType {
    Exact(usize),
    AtLeast(usize),
    AtMost(usize),
}
