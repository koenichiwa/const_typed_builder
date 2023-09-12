use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{info::FieldInfo, VecStreamResult, MANDATORY_PREFIX};

#[derive(Debug, Clone)]
pub struct FieldGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
}

impl<'a> FieldGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo]) -> Self {
        Self { fields }
    }

    pub fn data_struct_fields(&self) -> VecStreamResult {
        self.fields
            .iter()
            .map(|field| {
                let field_name = field.ident();

                let data_field_type = match field {
                    FieldInfo::Optional(field) => field.ty().to_token_stream(),
                    FieldInfo::Mandatory(field) if field.is_option_type() => {
                        field.ty().to_token_stream()
                    }
                    FieldInfo::Mandatory(field) => {
                        let ty = field.ty();
                        quote!(Option<#ty>)
                    }
                    FieldInfo::Grouped(field) => field.ty().to_token_stream(),
                };

                let tokens = quote!(
                    pub #field_name: #data_field_type
                );
                Ok(tokens)
            })
            .collect()
    }

    pub fn data_impl_fields(&self) -> VecStreamResult {
        self.fields
            .iter()
            .map(|field| {
                let field_name = field.ident();
                let tokens = match field {
                    FieldInfo::Mandatory(field) if field.is_option_type() => {
                        quote!(#field_name: data.#field_name)
                    }
                    FieldInfo::Optional(_) | FieldInfo::Grouped(_) => {
                        quote!(#field_name: data.#field_name)
                    }
                    FieldInfo::Mandatory(_) => {
                        quote!(#field_name: data.#field_name.unwrap())
                    }
                };
                Ok(tokens)
            })
            .collect()
    }

    pub fn target_impl_const_generics(&self) -> TokenStream {
        self.builder_const_generics_valued(false)
    }

    pub fn builder_impl_setters(&self, builder_name: &syn::Ident) -> VecStreamResult {
        self.fields
            .iter()
            .map(|field| {
                let const_idents_generic = self.builder_const_generic_idents_set_before(field);
                let const_idents_input = self.builder_const_generic_idents_set(field, false);
                let const_idents_output = self.builder_const_generic_idents_set(field, true);

                let field_name = field.ident();
                let input_type = self.builder_set_impl_input_type(field);
                let input_value = self.builder_set_impl_input_value(field);

                let tokens = quote!(
                    impl #const_idents_generic #builder_name #const_idents_input {
                        pub fn #field_name (self, #input_type) -> #builder_name #const_idents_output {
                            let mut data = self.data;
                            data.#field_name = #input_value;
                            #builder_name {
                                data,
                            }
                        }
                    }
                );
                Ok(tokens)
            })
            .collect()
    }

    fn builder_set_impl_input_type(&self, field: &FieldInfo) -> TokenStream {
        let field_name = field.ident();
        match field {
            FieldInfo::Optional(field) => {
                let ty = field.ty();
                quote!(#field_name: #ty)
            }
            FieldInfo::Mandatory(field) if field.is_option_type() => {
                let inner_ty = field.inner_type();
                quote!(#field_name: #inner_ty)
            }
            FieldInfo::Mandatory(field) => {
                let ty = field.ty();
                quote!(#field_name: #ty)
            }
            FieldInfo::Grouped(field) => {
                let inner_ty = field.inner_type();
                quote!(#field_name: #inner_ty)
            }
        }
    }

    fn builder_set_impl_input_value(&self, field: &FieldInfo) -> TokenStream {
        let field_name = field.ident();
        match field {
            FieldInfo::Optional(_) => quote!(#field_name),
            _ => quote!(Some(#field_name)),
        }
    }

    pub fn builder_const_generic_idents(&self) -> TokenStream {
        let all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(
                format_ident!("{}_{}", MANDATORY_PREFIX, mandatory.mandatory_index())
                    .to_token_stream(),
            ))
                as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index).into_token_stream()),
            ) as Box<dyn Iterator<Item = TokenStream>>,
        });
        quote!(<#(const #all: bool),*>)
    }

    pub fn builder_const_generics_valued(&self, value: bool) -> TokenStream {
        let all = self.fields.iter().flat_map(|field| match field {
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
        quote!(<#(#all),*>)
    }

    fn builder_const_generic_idents_set(&self, field_info: &FieldInfo, value: bool) -> TokenStream {
        let all = self.fields.iter().flat_map(|field| match field {
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
        quote!(<#(#all),*>)
    }

    fn builder_const_generic_idents_set_before(&self, field_info: &FieldInfo) -> TokenStream {
        let all =
            self.fields.iter().flat_map(|field| match field {
                FieldInfo::Optional(_) => Box::new(std::iter::empty::<TokenStream>())
                    as Box<dyn Iterator<Item = TokenStream>>,
                _ if field == field_info => Box::new(std::iter::empty::<TokenStream>())
                    as Box<dyn Iterator<Item = TokenStream>>,
                FieldInfo::Mandatory(field) => Box::new(std::iter::once(
                    format_ident!("{}_{}", MANDATORY_PREFIX, field.mandatory_index())
                        .into_token_stream(),
                ))
                    as Box<dyn Iterator<Item = TokenStream>>,
                FieldInfo::Grouped(field) => {
                    Box::new(field.group_indices().iter().map(|(group, index)| {
                        group.partial_const_ident(*index).into_token_stream()
                    })) as Box<dyn Iterator<Item = TokenStream>>
                }
            });
        quote!(<#(const #all: bool),*>)
    }

    pub fn builder_const_generic_idents_final(&self) -> TokenStream {
        let all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(_) => Box::new(std::iter::once(
                syn::LitBool::new(true, proc_macro2::Span::call_site()).into_token_stream(),
            )) as Box<dyn Iterator<Item = TokenStream>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index).into_token_stream()),
            ) as Box<dyn Iterator<Item = TokenStream>>,
        });
        quote!(<#(#all),*>)
    }

    pub fn builder_const_generic_group_partial_idents(&self) -> TokenStream {
        let all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Mandatory(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream>>
            }
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index).into_token_stream()),
            ) as Box<dyn Iterator<Item = TokenStream>>,
        });
        quote!(<#(const #all: bool),*>)
    }
}
