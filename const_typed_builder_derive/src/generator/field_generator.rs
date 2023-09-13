use crate::{info::FieldInfo, VecStreamResult};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub(super) struct FieldGenerator<'a> {
    fields: &'a [FieldInfo<'a>],
}

impl<'a> FieldGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo]) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[FieldInfo] {
        self.fields
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

    pub fn builder_set_impl_input_type(&self, field: &FieldInfo) -> TokenStream {
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

    pub fn builder_set_impl_input_value(&self, field: &FieldInfo) -> TokenStream {
        let field_name = field.ident();
        match field {
            FieldInfo::Optional(_) => quote!(#field_name),
            _ => quote!(Some(#field_name)),
        }
    }
}
