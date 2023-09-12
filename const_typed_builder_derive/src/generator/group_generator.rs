use proc_macro2::TokenStream;

use crate::info::{GroupInfo, GroupType};
use quote::{quote, ToTokens};

#[derive(Debug)]
pub struct GroupGenerator<'a> {
    groups: Vec<&'a GroupInfo>,
}

impl<'a> GroupGenerator<'a> {
    pub fn new(groups: Vec<&'a GroupInfo>) -> Self {
        Self { groups }
    }

    pub fn builder_build_impl_correctness_helper_fns(&self) -> TokenStream {
        if self.groups.is_empty() {
            return TokenStream::new();
        }

        let (exact, at_least, at_most) =
            self.groups
                .iter()
                .fold((false, false, false), |acc, group| {
                    match group.group_type() {
                        GroupType::Exact(_) => (true, acc.1, acc.2),
                        GroupType::AtLeast(_) => (acc.0, true, acc.2),
                        GroupType::AtMost(_) => (acc.0, acc.1, true),
                    }
                });

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

    pub fn builder_build_impl_correctness_check(&self) -> Option<TokenStream> {
        (!self.groups.is_empty()).then(|| quote!(let _ = Self::GROUP_VERIFIER;))
    }

    pub fn builder_build_impl_correctness_verifier(&self) -> Option<TokenStream> {
        if self.groups.is_empty() {
            return None;
        }

        let all = self.groups.iter().flat_map(|group| {
            let partials = (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream());
            let function_call = group.function_ident();
            let count = group.expected_count();
            let name = group.name();
            let err_text = format!("Group {name} not verified");

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
