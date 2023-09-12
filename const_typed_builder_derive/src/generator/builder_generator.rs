use proc_macro2::TokenStream;

use quote::quote;

use crate::StreamResult;

use super::{field_generator::FieldGenerator, group_generator::GroupGenerator};

pub struct BuilderGenerator<'a> {
    group_gen: GroupGenerator<'a>,
    field_gen: FieldGenerator<'a>,
    target_name: &'a syn::Ident,
    builder_name: &'a syn::Ident,
    data_name: &'a syn::Ident,
}

impl <'a> BuilderGenerator<'a> {
    pub fn new(
        group_gen: GroupGenerator<'a>,
        field_gen: FieldGenerator<'a>,
        target_name: &'a syn::Ident,
        builder_name: &'a syn::Ident,
        data_name: &'a syn::Ident
    ) -> Self {
        Self {
            group_gen,
            field_gen,
            target_name,
            builder_name,
            data_name,
        }
    }

    pub fn generate(&self) -> StreamResult {
        let builder_struct = self.generate_struct();
        let builder_impl = self.generate_impl()?;
        let tokens = quote!(
            #builder_struct
            #builder_impl
        );
        Ok(tokens)
    }

    fn generate_struct(&self) -> TokenStream {
        let data_name = self.data_name;
        let builder_name = self.builder_name;
        let const_idents = self.field_gen.builder_const_generic_idents();

        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name #const_idents {
                data: #data_name
            }
        )
    }

    fn generate_impl(&self) -> StreamResult {
        let builder_setters = self.generate_setters_impl()?;
        let builder_new = self.generate_new_impl();
        let builder_build = self.generate_build_impl();

        let tokens = quote!(
            #builder_new
            #builder_setters
            #builder_build
        );
        Ok(tokens)
    }

    fn generate_new_impl(&self) -> TokenStream {
        let builder_name = self.builder_name;
        let const_generics = self.field_gen.builder_const_generics_valued(false);

        quote!(
            impl #builder_name #const_generics {
                pub fn new() -> #builder_name #const_generics {
                    Self::default()
                }
            }
        )
    }

    fn generate_build_impl(&self) -> TokenStream {
        let target_name = self.target_name;
        let builder_name = self.builder_name;
        let group_partials = self.field_gen.builder_const_generic_group_partial_idents();
        let generic_consts = self.field_gen.builder_const_generic_idents_final();
        let correctness_verifier = self.group_gen.builder_build_impl_correctness_verifier();
        let correctness_check = self.group_gen.builder_build_impl_correctness_check();
        let correctness_helper_fns = self.group_gen.builder_build_impl_correctness_helper_fns();

        quote!(
            impl #group_partials #builder_name #generic_consts {
                #correctness_verifier
                #correctness_helper_fns

                pub fn build(self) -> #target_name {
                    #correctness_check
                    self.data.into()
                }
            }
        )
    }

    fn generate_setters_impl(&self) -> StreamResult {
        let setters = self.field_gen.builder_impl_setters(self.builder_name)?;

        let tokens = quote!(
            #(#setters)*
        );
        Ok(tokens)
    }
}