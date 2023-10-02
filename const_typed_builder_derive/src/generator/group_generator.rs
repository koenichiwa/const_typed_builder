use crate::{
    info::{GroupInfo, GroupType},
    CONST_IDENT_PREFIX,
};
use itertools::{Itertools, Powerset};
use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::{format_ident, quote};
use std::{collections::BTreeSet, cmp::Ordering};

/// The `GroupGenerator` struct is responsible for generating code related to groups within the builder, including correctness checks and verifications.
#[derive(Debug)]
pub(super) struct GroupGenerator<'a> {
    groups: Vec<&'a GroupInfo>,
}

impl<'a> GroupGenerator<'a> {
    /// Creates a new `GroupGenerator` instance.
    ///
    /// # Arguments
    ///
    /// - `groups`: A vector of references to `GroupInfo` representing the groups associated with the builder.
    ///
    /// # Returns
    ///
    /// A `GroupGenerator` instance initialized with the provided groups.
    pub fn new(groups: Vec<&'a GroupInfo>) -> Self {
        Self::can_be_valid(&groups);
        Self { groups }
    }

    fn can_be_valid(groups: &[&'a GroupInfo]) {
        groups.iter().for_each(|group| {
            let valid_range = 1..group.indices().len();
            if valid_range.is_empty() {
                emit_warning!(group.name(), format!("There is not an valid expected count"))
            } else if !valid_range.contains(&group.expected_count()) {
                emit_warning!(group.name(), format!("Expected count is outside of valid range {valid_range:#?}"));
            }
            match group.group_type() {
                GroupType::Exact(expected) => {
                    match expected.cmp(&valid_range.start) {
                        Ordering::Less => emit_error!(group.name(), "Group can never be satisfied"),
                        Ordering::Equal | Ordering::Greater => {},
                    }
                    match expected.cmp(&valid_range.end) {
                        Ordering::Less => {}
                        Ordering::Equal => emit_warning!(group.name(), "Group can only be satisfied if all fields are initialized"; hint = "Consider removing group and using [builder(mandatory)] instead"),
                        Ordering::Greater => emit_error!(group.name(), format!("Group can never be satisfied. Need exact {} out of {} fields", expected, valid_range.end)),
                    }
                },
                GroupType::AtLeast(expected) => {
                    match expected.cmp(&valid_range.start) {
                        Ordering::Less => emit_warning!(group.name(), "Group has no effect"; hint = "Consider removing the group"),
                        Ordering::Equal | Ordering::Greater => {},
                    }
                    match expected.cmp(&valid_range.end) {
                        Ordering::Less => {}
                        Ordering::Equal => emit_warning!(group.name(), "Group can only be satisfied if all fields are initialized"; hint = "Consider removing group and using [builder(mandatory)] instead"),
                        Ordering::Greater => emit_error!(group.name(), format!("Group can never be satisfied. Need at least {} out of {} fields", expected, valid_range.end)),
                    }
                },
                GroupType::AtMost(expected) => {
                    match expected.cmp(&valid_range.start) {
                        Ordering::Less => emit_error!(group.name(), "This group prevents all of the fields to be initialized"; hint = "Remove the group and use [builder(skip)] instead"),
                        Ordering::Equal | Ordering::Greater => {},
                    }
                    match expected.cmp(&valid_range.end) {
                        Ordering::Less => {}
                        Ordering::Equal | Ordering::Greater => emit_warning!(group.name(), "Group has no effect"; hint = "Consider removing the group"),
                    }
                },
            }
        });
    }

    /// Returns all valid combinations of the const generics for the grouped fields
    ///
    /// # Returns
    ///
    /// An `Iterator<Item = Vec<usize>>` that holds each vector of field indices
    pub fn valid_groupident_combinations(&self) -> impl Iterator<Item = Vec<usize>> + '_ {
        let group_indices: BTreeSet<usize> = self
            .groups
            .iter()
            .flat_map(|group| group.indices().clone())
            .collect();
        let powerset: Powerset<std::collections::btree_set::IntoIter<usize>> =
            group_indices.into_iter().powerset();
        powerset.filter_map(|set| {
            if self.groups.iter().all(|group| group.is_valid_with(&set)) {
                Some(set)
            } else {
                None
            }
        })
    }

    /// Generates correctness helper functions for group validation and returns a `TokenStream`.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated correctness helper functions.
    pub fn builder_build_impl_correctness_helper_fns(&self) -> TokenStream {
        if self.groups.is_empty() {
            return TokenStream::new();
        }

        let mut exact = false;
        let mut at_least = false;
        let mut at_most = false;

        for group in &self.groups {
            match group.group_type() {
                GroupType::Exact(_) => exact = true,
                GroupType::AtLeast(_) => at_least = true,
                GroupType::AtMost(_) => at_most = true,
            }

            if exact && at_least && at_most {
                break;
            }
        }

        let exact = exact.then(|| {
            quote!(
                const fn exact(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count == count
                }
            )
        });

        let at_least = at_least.then(|| {
            quote!(
                const fn at_least(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count >= count
                }
            )
        });

        let at_most = at_most.then(|| {
            quote!(
                const fn at_most(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count <= count
                }
            )
        });
        quote!(
            #exact
            #at_least
            #at_most
        )
    }

    /// Generates the correctness check for groups and returns a `TokenStream` as an optional value.
    ///
    /// # Returns
    ///
    /// An optional `TokenStream` representing the generated correctness check. Returns `None` if there are no groups.
    pub fn builder_build_impl_correctness_check(&self) -> Option<TokenStream> {
        (!self.groups.is_empty()).then(|| quote!(let _ = Self::GROUP_VERIFIER;))
    }

    /// Generates the correctness verifier for groups and returns an optional `TokenStream`.
    ///
    /// # Returns
    ///
    /// An optional `TokenStream` representing the generated correctness verifier. Returns `None` if there are no groups.
    pub fn builder_build_impl_correctness_verifier(&self) -> Option<TokenStream> {
        if self.groups.is_empty() {
            return None;
        }

        let all = self.groups.iter().map(|group| {
            let partials = group.indices().iter().map(|index| format_ident!("{}{}", CONST_IDENT_PREFIX, index));
            let function_call: syn::Ident = group.function_symbol().into();
            let count = group.expected_count();
            let name = group.name();
            let function_name = group.function_symbol().to_string();
            let err_text = format!("`.build()` failed because the bounds of group `{name}` where not met ({function_name} {count})");

            quote!(
                if !Self::#function_call(&[#(#partials),*], #count) {
                    panic!(#err_text);
                }
            )
        });
        Some(quote!(
            const GROUP_VERIFIER: ()  = {
                #(#all)*
                ()
            };
        ))
    }
}
