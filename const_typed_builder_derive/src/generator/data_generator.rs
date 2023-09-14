use super::{field_generator::FieldGenerator, generics_generator::GenericsGenerator};
use crate::StreamResult;
use quote::quote;

/// The `DataGenerator` struct is responsible for generating code related to the data struct
/// that corresponds to the target struct and the conversion implementations.
pub(super) struct DataGenerator<'a> {
    field_gen: FieldGenerator<'a>,
    generics_gen: GenericsGenerator<'a>,
    target_name: &'a syn::Ident,
    data_name: &'a syn::Ident,
}

impl<'a> DataGenerator<'a> {
    /// Creates a new `DataGenerator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `field_gen`: The `FieldGenerator` responsible for generating field-related code.
    /// - `generics_gen`: The `GenericsGenerator` responsible for generating generics information.
    /// - `target_name`: A reference to the identifier representing the target struct's name.
    /// - `data_name`: A reference to the identifier representing the data struct's name.
    ///
    /// # Returns
    ///
    /// A `DataGenerator` instance initialized with the provided information.
    pub(super) fn new(
        field_gen: FieldGenerator<'a>,
        generics_gen: GenericsGenerator<'a>,
        target_name: &'a syn::Ident,
        data_name: &'a syn::Ident,
    ) -> Self {
        Self {
            field_gen,
            generics_gen,
            target_name,
            data_name,
        }
    }

    /// Generates the code for the data struct and the conversion implementations and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `StreamResult` representing the generated code for the data struct and conversions.
    pub fn generate(&self) -> StreamResult {
        let data_struct = self.generate_struct()?;
        let data_impl = self.generate_impl()?;

        let tokens = quote!(
            #data_struct
            #data_impl
        );

        Ok(tokens)
    }

    /// Generates the implementation code for conversions between the data struct and the target struct.
    fn generate_impl(&self) -> StreamResult {
        let data_name = self.data_name;
        let struct_name = self.target_name;
        let from_fields = self.field_gen.data_impl_from_fields()?;
        let def_fields = self.field_gen.data_impl_default_fields();

        let (impl_generics, type_generics, where_clause) =
            self.generics_gen.target_generics().split_for_impl();

        let tokens = quote!(
            impl #impl_generics From<#data_name #type_generics> for #struct_name #type_generics #where_clause {
                fn from(data: #data_name #type_generics) -> #struct_name #type_generics {
                    #struct_name {
                        #(#from_fields),*
                    }
                }
            }

            impl #impl_generics Default for #data_name #type_generics #where_clause {
                fn default() -> Self {
                    #data_name {
                        #def_fields
                    }
                }
            }
        );
        Ok(tokens)
    }

    /// Generates the code for the data struct itself.
    fn generate_struct(&self) -> StreamResult {
        let data_name = self.data_name;

        let fields = self.field_gen.data_struct_fields()?;
        let (impl_generics, _type_generics, where_clause) =
            self.generics_gen.target_generics().split_for_impl();

        let tokens = quote!(
            #[derive(Debug)]
            pub struct #data_name #impl_generics #where_clause{
                #(#fields),*
            }
        );
        Ok(tokens)
    }
}
