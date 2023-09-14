use proc_macro2::TokenStream;
use quote::quote;
use super::generics_generator::GenericsGenerator;

/// The `TargetGenerator` struct is responsible for generating code for the target struct implementation
/// of the builder pattern based on the provided `GenericsGenerator`, target name, and builder name.
pub(super) struct TargetGenerator<'a> {
    generics_gen: GenericsGenerator<'a>,
    target_name: &'a syn::Ident,
    builder_name: &'a syn::Ident,
}

impl<'a> TargetGenerator<'a> {
    /// Creates a new `TargetGenerator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `generics_gen`: The `GenericsGenerator` responsible for generating generics information.
    /// - `target_name`: A reference to the identifier representing the target struct's name.
    /// - `builder_name`: A reference to the identifier representing the builder struct's name.
    ///
    /// # Returns
    ///
    /// A `TargetGenerator` instance initialized with the provided information.
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

    /// Generates the target struct's builder implementation code and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated code for the builder implementation.
    pub fn generate(&self) -> TokenStream {
        self.generate_impl()
    }

    /// Generates the actual implementation code for the target struct.
    fn generate_impl(&self) -> TokenStream {
        let target_name = self.target_name;
        let builder_name = self.builder_name;
        let const_generics = self.generics_gen.const_generics_valued(false);
        let _builder_generics = self.generics_gen.builder_struct_generics();
        let (impl_generics, type_generics, where_clause) =
            self.generics_gen.target_generics().split_for_impl();

        quote! {
            impl #impl_generics Builder for #target_name #type_generics #where_clause {
                type BuilderImpl = #builder_name #const_generics;

                fn builder() -> Self::BuilderImpl  {
                    Self::BuilderImpl::new()
                }
            }
        }
    }
}
