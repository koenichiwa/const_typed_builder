use crate::{
    info::{FieldInfo, FieldKind},
    VecStreamResult,
};
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

                let data_field_type = match field.kind() {
                    FieldKind::Optional => field.ty().to_token_stream(),
                    FieldKind::Mandatory if field.is_option_type() => field.ty().to_token_stream(),
                    FieldKind::Mandatory => {
                        let ty = field.ty();
                        quote!(Option<#ty>)
                    }
                    FieldKind::Grouped => field.ty().to_token_stream(),
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
                let tokens = match field.kind() {
                    FieldKind::Mandatory if field.is_option_type() => {
                        quote!(#field_name: data.#field_name)
                    }
                    FieldKind::Optional | FieldKind::Grouped => {
                        quote!(#field_name: data.#field_name)
                    }
                    FieldKind::Mandatory => {
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
        match field.kind() {
            FieldKind::Optional => field.ty(),
            FieldKind::Mandatory if field.is_option_type() => field.inner_type().expect(
                "Couldn't read inner type of option, even though it's seen as an Option type",
            ),
            FieldKind::Mandatory => field.ty(),
            FieldKind::Grouped => field
                .inner_type()
                .expect("Couldn't read inner type of option, even though it's marked as grouped"),
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

        match field.kind() {
            FieldKind::Optional => quote!(#field_value),
            _ => quote!(Some(#field_value)),
        }
    }
}
