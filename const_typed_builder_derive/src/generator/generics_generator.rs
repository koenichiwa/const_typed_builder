use crate::{info::FieldInfo, MANDATORY_PREFIX};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

#[derive(Debug, Clone)]
pub(super) struct GenericsGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
    target_generics: &'a syn::Generics,
}

impl<'a> GenericsGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo], target_generics: &'a syn::Generics) -> Self {
        Self {
            fields,
            target_generics,
        }
    }

    pub fn target_generics(&self) -> &syn::Generics {
        &self.target_generics
    }

    pub fn target_impl_const_generics(&self) -> TokenStream {
        self.builder_const_generics_valued(false)
    }

    pub fn builder_impl_new_generics(&self) -> TokenStream {
        self.builder_const_generics_valued(false)
    }

    pub fn builder_const_generics_valued(&self, value: bool) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(
                syn::LitBool::new(value, mandatory.ident().span()).to_token_stream(),
            ))
                as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) => {
                Box::new(grouped.group_indices().iter().map(|(_, _)| {
                    syn::LitBool::new(value, grouped.ident().span()).into_token_stream()
                })) as Box<dyn Iterator<Item = TokenStream>>
            }
        });
        self.add_const_generics_valued(&mut all)
    }

    pub fn builder_const_generic_idents_set_after(
        &self,
        field_info: &FieldInfo,
        value: bool,
    ) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(_) if field_info == field => Box::new(std::iter::once(
                syn::LitBool::new(value, field_info.ident().span()).into_token_stream(),
            ))
                as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(
                format_ident!("{}_{}", MANDATORY_PREFIX, mandatory.mandatory_index())
                    .into_token_stream(),
            ))
                as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) if field_info == field => Box::new(
                std::iter::repeat(
                    syn::LitBool::new(value, grouped.ident().span()).into_token_stream(),
                )
                .take(grouped.group_indices().len()),
            )
                as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index).into_token_stream()),
            ) as Box<dyn Iterator<Item = TokenStream>>,
        });
        self.add_const_generics_valued(&mut all)
    }

    pub fn builder_const_generic_idents_set_before(&self, field_info: &FieldInfo) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            _ if field == field_info => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(field) => Box::new(std::iter::once(format_ident!(
                "{}_{}",
                MANDATORY_PREFIX,
                field.mandatory_index()
            ))) as Box<dyn Iterator<Item = syn::Ident>>,
            FieldInfo::Grouped(field) => Box::new(
                field
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics(&mut all)
    }

    pub fn builder_const_generic_idents_build(&self) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(_) => Box::new(std::iter::once(
                syn::LitBool::new(true, proc_macro2::Span::call_site()).into_token_stream(), // FIXME
            )) as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index).into_token_stream()),
            ) as Box<dyn Iterator<Item = TokenStream>>,
        });
        self.add_const_generics_valued(&mut all)
    }

    pub fn builder_const_generic_group_partial_idents(&self) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics(&mut all)
    }

    pub fn builder_struct_generics(&self) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(format_ident!(
                "M_{}",
                mandatory.mandatory_index()
            )))
                as Box<dyn Iterator<Item = syn::Ident>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics(&mut all)
    }

    fn add_const_generics(&self, tokens: &mut dyn Iterator<Item = syn::Ident>) -> syn::Generics {
        let mut res = self.target_generics.clone();

        let syn::Generics { ref mut params, .. } = res;
        let before = dbg!(params.len());
        let mut count = 0;
        tokens.for_each(|token| {
            count += 1;
            params.push(parse_quote!(const #token: bool));
        });
        let after = dbg!(params.len());
        dbg!(count);
        assert_eq!(before + count, after);
        res
    }

    fn add_const_generics_valued(
        &self,
        tokens: &mut dyn Iterator<Item = TokenStream>,
    ) -> TokenStream {
        let syn::Generics { params, .. } = self.target_generics;
        if params.is_empty() {
            quote!(<#(#tokens),*>)
        } else {
            let type_generics = params.iter();
            quote!(< #(#type_generics),*, #(#tokens),* >)
        }
    }
}
