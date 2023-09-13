use crate::{info::FieldInfo, VecStreamResult};
use proc_macro2::TokenStream;
use quote::{ quote, ToTokens};

use super::generics_generator::GenericsGenerator;

#[derive(Debug, Clone)]
pub(super) struct FieldGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
    generics_gen: GenericsGenerator<'a>,
}

impl<'a> FieldGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo], generics_gen: GenericsGenerator<'a>,) -> Self {
        Self {
            fields,
            generics_gen,
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

    pub fn builder_impl_setters(
        &self,
        builder_name: &syn::Ident,
    ) -> VecStreamResult {
        self.fields
            .iter()
            .map(|field| {
                let const_idents_generic = self.generics_gen.builder_const_generic_idents_set_before(field);
                let const_idents_input = self.generics_gen.builder_const_generic_idents_set_after(field, false);
                let const_idents_output = self.generics_gen.builder_const_generic_idents_set_after(field, true);

                let field_name = field.ident();
                let input_type = self.builder_set_impl_input_type(field);
                let input_value = self.builder_set_impl_input_value(field);
                let where_clause = &self.generics_gen.target_generics().where_clause;

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
}
