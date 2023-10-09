use super::util;
use crate::info;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// The `TargetGenerator` struct is responsible for generating code for the target struct implementation
/// of the builder pattern based on the provided `info::Container`.
pub struct TargetGenerator<'info> {
    info: &'info info::Container<'info>,
}

impl<'info> TargetGenerator<'info> {
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
    pub fn new(info: &'info info::Container<'info>) -> Self {
        Self { info }
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
        let target_ident = self.info.ident();
        let builder_ident = self.info.builder_ident();

        let builder_impl = if self.info.generate_module() {
            let mod_ident = self.info.mod_ident();
            quote!(#mod_ident::#builder_ident)
        } else {
            builder_ident.to_token_stream()
        };

        let const_generics = util::const_generics_all_valued(false, self.info.field_collection(), self.info.generics());
        let (impl_generics, type_generics, where_clause) = self.info.generics().split_for_impl();

        let documentation = format!("Creates an instance of [`{}`]", self.info.builder_ident());
        quote! {
            impl #impl_generics Builder for #target_ident #type_generics #where_clause {
                type BuilderImpl = #builder_impl #const_generics;

                #[doc = #documentation]
                fn builder() -> Self::BuilderImpl  {
                    Self::BuilderImpl::new()
                }
            }
        }
    }
}
