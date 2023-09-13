use proc_macro2::TokenStream;

use quote::quote;

use super::generics_generator::GenericsGenerator;

pub(super) struct TargetGenerator<'a> {
    generics_gen: GenericsGenerator<'a>,
    target_name: &'a syn::Ident,
    builder_name: &'a syn::Ident,
}

impl<'a> TargetGenerator<'a> {
    pub fn new(
        generics_gen: GenericsGenerator<'a>,
        target_name: &'a syn::Ident,
        builder_name: &'a syn::Ident,
    ) -> Self {
        Self {
            generics_gen,
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
        let const_generics = self.generics_gen.const_generics_valued(false);
        let (impl_generics, type_generics, where_clause) =
            self.generics_gen.target_generics().split_for_impl();

        quote! {
            impl #impl_generics #target_name #type_generics #where_clause {
                pub fn builder() -> #builder_name #const_generics {
                    #builder_name::new()
                }
            }
        }
    }
}
