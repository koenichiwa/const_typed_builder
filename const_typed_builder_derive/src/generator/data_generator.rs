use crate::info::{Container, FieldKind};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// The `DataGenerator` struct is responsible for generating code related to the data struct
/// that corresponds to the target struct and the conversion implementations.
pub struct DataGenerator<'a> {
    info: &'a Container<'a>,
}

impl<'a> DataGenerator<'a> {
    /// Creates a new `DataGenerator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `info`: The `Container` containing all the information of the data container.
    ///
    /// # Returns
    ///
    /// A `DataGenerator` instance initialized with the provided information.
    pub fn new(info: &'a Container<'a>) -> Self {
        Self { info }
    }

    /// Generates the code for the data struct and the conversion implementations and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated code for the data struct and conversions.
    pub fn generate(&self) -> TokenStream {
        let data_struct = self.generate_struct();
        let data_impl = self.generate_impl();

        let tokens = quote!(
            #data_struct
            #data_impl
        );

        tokens
    }

    /// Generates the implementation code for conversions between the data struct and the target struct.
    fn generate_impl(&self) -> TokenStream {
        let data_ident = self.info.data_ident();
        let struct_ident = self.info.ident();
        let from_fields = self.impl_from_fields();
        let def_fields = self.impl_default_fields();

        let (impl_generics, type_generics, where_clause) = self.info.generics().split_for_impl();

        let tokens = quote!(
            impl #impl_generics From<#data_ident #type_generics> for #struct_ident #type_generics #where_clause {
                #[doc(hidden)]
                fn from(data: #data_ident #type_generics) -> #struct_ident #type_generics {
                    #struct_ident {
                        #(#from_fields),*
                    }
                }
            }

            impl #impl_generics Default for #data_ident #type_generics #where_clause {
                #[doc(hidden)]
                fn default() -> Self {
                    #data_ident {
                        #def_fields
                    }
                }
            }
        );
        tokens
    }

    /// Generates the code for the data struct itself.
    fn generate_struct(&self) -> TokenStream {
        let data_ident = self.info.data_ident();

        let fields = self.struct_fields();
        let (impl_generics, _type_generics, where_clause) = self.info.generics().split_for_impl();

        let tokens = quote!(
            #[doc(hidden)]
            pub struct #data_ident #impl_generics #where_clause{
                #(#fields),*
            }
        );
        tokens
    }

    /// Generates code for the fields of the data struct and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `Vec<TokenStream>` representing the data struct fields: `pub field_ident: field_type`.
    fn struct_fields(&self) -> Vec<TokenStream> {
        self.info
            .field_collection()
            .iter()
            .filter_map(|field| {
                let field_ident = field.ident();

                let data_field_type = match field.kind() {
                    FieldKind::Skipped => return None,
                    FieldKind::Optional => field.ty().to_token_stream(),
                    FieldKind::Mandatory if field.is_option_type() => field.ty().to_token_stream(),
                    FieldKind::Mandatory => {
                        let ty = field.ty();
                        quote!(Option<#ty>)
                    }
                    FieldKind::Grouped => field.ty().to_token_stream(),
                };

                let tokens = quote!(
                    pub #field_ident: #data_field_type
                );
                Some(tokens)
            })
            .collect()
    }

    // Generates code for the `From` trait implementation for converting data struct fields to target struct fields and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `Vec<TokenStream>` representing the fields for the `From` trait implementation. Either containing `unwrap`, `None` or just the type.
    fn impl_from_fields(&self) -> Vec<TokenStream> {
        self.info
            .field_collection()
            .iter()
            .map(|field| {
                let field_ident = field.ident();
                let tokens = match field.kind() {
                    FieldKind::Skipped => quote!(#field_ident: None),
                    FieldKind::Mandatory if field.is_option_type() => {
                        quote!(#field_ident: data.#field_ident)
                    }
                    FieldKind::Optional | FieldKind::Grouped => {
                        quote!(#field_ident: data.#field_ident)
                    }
                    FieldKind::Mandatory => {
                        quote!(#field_ident: data.#field_ident.unwrap())
                    }
                };
                tokens
            })
            .collect()
    }

    /// Generates default field values for the data struct and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated default field values.
    fn impl_default_fields(&self) -> TokenStream {
        let fields_none = self
            .info
            .field_collection()
            .iter()
            .filter(|field| field.kind() != FieldKind::Skipped)
            .map(|field| {
                let field_ident = field.ident();
                quote!(#field_ident: None)
            });
        quote!(
            #(#fields_none),*
        )
    }
}
