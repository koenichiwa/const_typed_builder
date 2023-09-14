use crate::info::{GroupInfo, GroupType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

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
        Self { groups }
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

        let all = self.groups.iter().flat_map(|group| {
            let partials = (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream());
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