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

    fn field_effective_type(field: &'a FieldInfo) -> &'a syn::Type {
        match field {
            FieldInfo::Optional(field) => field.ty(),
            FieldInfo::Mandatory(field) if field.is_option_type() => field
                .inner_type()
                .expect("Couldn't read inner type of option, even though it's marked as optional"),
            FieldInfo::Mandatory(field) => field.ty(),
            FieldInfo::Grouped(field) => field.inner_type(),
        }
    }

    pub fn builder_set_impl_input_type(&self, field: &'a FieldInfo) -> TokenStream {
        let field_name = field.ident();
        let field_ty = Self::field_effective_type(field);
        let bottom_ty = if field.is_option_type() {
            field.inner_type().unwrap()
        } else {
            field_ty
        };

        let field_ty = if field.propagate() {
            quote!(fn(<#bottom_ty as Builder>:: BuilderImpl) -> #field_ty)
        } else {
            quote!(#field_ty)
        };

        quote!(#field_name: #field_ty)
    }

    pub fn builder_set_impl_input_value(&self, field: &'a FieldInfo) -> TokenStream {
        let field_name = field.ident();

        let field_ty = Self::field_effective_type(field);
        let bottom_ty = if field.is_option_type() {
            field.inner_type().unwrap()
        } else {
            field_ty
        };

        let field_value = if field.propagate() {
            quote!(#field_name(<#bottom_ty as Builder>::builder()))
        } else {
            quote!(#field_name)
        };

        match field {
            FieldInfo::Optional(_) => quote!(#field_value),
            _ => quote!(Some(#field_value)),
        }
    }
}
