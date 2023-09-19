use crate::{
    info::{FieldInfo, FieldKind},
    VecStreamResult,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// The `FieldGenerator` struct is responsible for generating code related to fields of the target and data structs.
#[derive(Debug, Clone)]
pub(super) struct FieldGenerator<'a> {
    fields: &'a [FieldInfo<'a>],
}

impl<'a> FieldGenerator<'a> {
    /// Creates a new `FieldGenerator` instance.
    ///
    /// # Arguments
    ///
    /// - `fields`: A reference to a slice of `FieldInfo` representing the fields of the struct.
    ///
    /// # Returns
    ///
    /// A `FieldGenerator` instance initialized with the provided fields.
    pub fn new(fields: &'a [FieldInfo]) -> Self {
        Self { fields }
    }

    /// Returns a reference to the fields of the struct.
    pub fn fields(&self) -> &[FieldInfo] {
        self.fields
    }

    /// Generates code for the fields of the data struct and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `VecStreamResult` representing the generated code for the data struct fields.
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

    // Generates code for the `From` trait implementation for converting data struct fields to target struct fields and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `VecStreamResult` representing the generated code for the `From` trait implementation.
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

    /// Generates default field values for the data struct and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated default field values.
    pub fn data_impl_default_fields(&self) -> TokenStream {
        let fields_none = self.fields.iter().map(|field| {
            let field_name = field.ident();
            quote!(#field_name: None)
        });
        quote!(
            #(#fields_none),*
        )
    }

    /// Generates code for the input type of a builder setter method and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `field`: A reference to the `FieldInfo` for which the setter input type is generated.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated input type for the builder setter method.
    pub fn builder_set_impl_input_type(&self, field: &'a FieldInfo) -> TokenStream {
        let field_name = field.ident();
        let field_ty = field.setter_input_type();
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

    /// Generates code for the input value of a builder setter method and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `field`: A reference to the `FieldInfo` for which the setter input value is generated.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated input value for the builder setter method.
    pub fn builder_set_impl_input_value(&self, field: &'a FieldInfo) -> TokenStream {
        let field_name = field.ident();

        let field_ty = field.setter_input_type();
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
