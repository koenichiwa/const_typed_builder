use super::field_generator::FieldGenerator;
use crate::StreamResult;
use quote::quote;

pub(super) struct DataGenerator<'a> {
    field_gen: FieldGenerator<'a>,
    target_name: &'a syn::Ident,
    data_name: &'a syn::Ident,
}

impl<'a> DataGenerator<'a> {
    pub(super) fn new(
        field_gen: FieldGenerator<'a>,
        target_name: &'a syn::Ident,
        data_name: &'a syn::Ident,
    ) -> Self {
        Self {
            field_gen,
            target_name,
            data_name,
        }
    }

    pub fn generate(&self) -> StreamResult {
        let data_struct = self.generate_struct()?;
        let data_impl = self.generate_impl()?;

        let tokens = quote!(
            #data_struct
            #data_impl
        );

        Ok(tokens)
    }

    fn generate_impl(&self) -> StreamResult {
        let data_name = self.data_name;
        let struct_name = self.target_name;
        let fields = self.field_gen.data_impl_fields()?;
        
        let (impl_generics, type_generics, where_clause) = self.field_gen.target_generics().split_for_impl();

        let tokens = quote!(
            impl #impl_generics From<#data_name #type_generics> for #struct_name #type_generics #where_clause {
                fn from(data: #data_name #type_generics) -> #struct_name #type_generics {
                    #struct_name {
                        #(#fields),*
                    }
                }
            }
        );
        Ok(tokens)
    }

    fn generate_struct(&self) -> StreamResult {
        let data_name = self.data_name;

        let fields = self.field_gen.data_struct_fields()?;
        let (impl_generics, type_generics, where_clause) = self.field_gen.target_generics().split_for_impl();

        let tokens = quote!(
            #[derive(Default, Debug)]
            pub struct #data_name #impl_generics #where_clause{
                #(#fields),*
            }
        );
        Ok(tokens)
    }
}
