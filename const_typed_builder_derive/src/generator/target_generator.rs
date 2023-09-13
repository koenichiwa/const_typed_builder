use proc_macro2::TokenStream;

use quote::quote;

use super::field_generator::FieldGenerator;

pub(super) struct TargetGenerator<'a> {
    field_gen: FieldGenerator<'a>,
    target_name: &'a syn::Ident,
    builder_name: &'a syn::Ident,
}

impl<'a> TargetGenerator<'a> {
    pub fn new(
        field_gen: FieldGenerator<'a>,
        target_name: &'a syn::Ident,
        builder_name: &'a syn::Ident,
    ) -> Self {
        Self {
            field_gen,
            target_name,
            builder_name,
        }
    }

    pub fn generate(&self) -> TokenStream {
        self.generate_impl()
    }

    fn generate_impl(&self) -> TokenStream {
        let target_name = self.target_name;
        let builder_name = self.builder_name;
        let const_generics = self.field_gen.target_impl_const_generics();
        let (impl_generics, type_generics, where_clause) =
            self.field_gen.target_generics().split_for_impl();

        quote! {
            impl #impl_generics #target_name #type_generics #where_clause {
                pub fn builder() -> #builder_name #const_generics {
                    #builder_name::new()
                }
            }
        }
    }
}
