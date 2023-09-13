use crate::{info::FieldInfo, VecStreamResult, MANDATORY_PREFIX};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

#[derive(Debug, Clone)]
pub(super) struct FieldGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
    target_generics: &'a syn::Generics,
}

impl<'a> FieldGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo], target_generics: &'a syn::Generics) -> Self {
        Self {
            fields,
            target_generics,
        }
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

    pub fn data_impl_from_fields(&self) -> VecStreamResult {
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

    pub fn data_impl_default_fields(&self) -> TokenStream {
        let fields_none = self.fields.iter().map(|field| {
            let field_name = field.ident();
            quote!(#field_name: None)
        });
        quote!(
            #(#fields_none),*
        )
    }

    pub fn target_impl_const_generics(&self) -> TokenStream {
        self.builder_const_generics_valued(false)
    }

    pub fn builder_impl_new_generics(&self) -> TokenStream {
        self.builder_const_generics_valued(false)
    }

    pub fn builder_impl_setters(
        &self,
        builder_name: &syn::Ident,
    ) -> VecStreamResult {
        self.fields
            .iter()
            .map(|field| {
                let const_idents_generic = self.builder_const_generic_idents_set_before(field);
                let const_idents_input = self.builder_const_generic_idents_set(field, false);
                let const_idents_output = self.builder_const_generic_idents_set(field, true);

                let field_name = field.ident();
                let input_type = self.builder_set_impl_input_type(field);
                let input_value = self.builder_set_impl_input_value(field);

                let where_clause = &self.target_generics.where_clause;

                let tokens = quote!(
                    impl #const_idents_generic #builder_name #const_idents_input #where_clause {
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
        // quote!(#(#all),*)
        self.add_const_generics_valued(&mut all)
    }

    fn builder_const_generic_idents_set(&self, field_info: &FieldInfo, value: bool) -> TokenStream {
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
        // quote!(<#(#all),*>)
    }

    fn builder_const_generic_idents_set_before(&self, field_info: &FieldInfo) -> syn::Generics {
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
        // Self::combine_generics(quote!(#(const #all: bool),*), target_generics)
    }

    pub fn builder_const_generic_idents_final(&self) -> TokenStream {
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
        // Self::combine_generics(quote!(#(#all),*), target_generics)
        // Self::add_const_generics(&all, target_generics)
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
        // self.target_generics.clone()
        // Self::combine_generics(quote!(#(const #all: bool),*), target_generics)
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

    pub fn add_const_generics_valued(
        &self,
        tokens: &mut dyn Iterator<Item = TokenStream>,
    ) -> TokenStream {
        let syn::Generics {
            params,
            ..
        } = self.target_generics;
        if params.is_empty() {
            quote!(<#(#tokens),*>)
        } else {
            let type_generics = params.iter();
            quote!(< #(#type_generics),*, #(#tokens),* >)
        }

        // quote!(< #type_generics, #(#tokens),* >)
        // quote!(<#type_generics>)
    }

    pub fn target_generics(&self) -> &syn::Generics {
        &self.target_generics
    }
}
