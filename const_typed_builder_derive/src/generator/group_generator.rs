use crate::{
    info::{GroupInfo, GroupType},
    CONST_IDENT_PREFIX,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Debug)]
pub(super) struct GroupGenerator<'a> {
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

    pub fn builder_build_impl_correctness_check(&self) -> Option<TokenStream> {
        (!self.groups.is_empty()).then(|| quote!(let _ = Self::GROUP_VERIFIER;))
    }

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
