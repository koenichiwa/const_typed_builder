use super::{field_generator::FieldGenerator, group_generator::GroupGenerator, generics_generator::GenericsGenerator};
use crate::{StreamResult, VecStreamResult};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) struct BuilderGenerator<'a> {
    group_gen: GroupGenerator<'a>,
    field_gen: FieldGenerator<'a>,
    generics_gen: GenericsGenerator<'a>,
    target_name: &'a syn::Ident,
    target_vis: &'a syn::Visibility,
    builder_name: &'a syn::Ident,
    data_name: &'a syn::Ident,
}

impl<'a> BuilderGenerator<'a> {
    pub fn new(
        group_gen: GroupGenerator<'a>,
        field_gen: FieldGenerator<'a>,
        generics_gen: GenericsGenerator<'a>,
        target_name: &'a syn::Ident,
        target_vis: &'a syn::Visibility,
        builder_name: &'a syn::Ident,
        data_name: &'a syn::Ident,
    ) -> Self {
        Self {
            group_gen,
            field_gen,
            generics_gen,
            target_name,
            target_vis,
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
        let generics = self.generics_gen.builder_struct_generics();
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let (_, type_generics, _) = self.generics_gen.target_generics().split_for_impl();

        let vis = self.target_vis;

        quote!(
            #[derive(Debug)]
            #vis struct #builder_name #impl_generics #where_clause {
                data: #data_name #type_generics
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
        let data_name = self.data_name;
        // let const_generics = self.field_gen.builder_const_generics_valued(false);
        // let generics = self.field_gen.builder_struct_generics();

        let type_generics = self.generics_gen.builder_impl_new_generics();
        let (impl_generics, _, where_clause) = self.generics_gen.target_generics().split_for_impl();

        quote!(
            impl #impl_generics #builder_name #type_generics #where_clause{
                pub fn new() -> #builder_name #type_generics {
                    Self::default()
                }
            }

            impl #impl_generics Default for #builder_name #type_generics #where_clause {
                fn default() -> Self {
                    #builder_name {
                        data: #data_name::default(),
                    }
                }
            }
        )
    }

    fn generate_build_impl(&self) -> TokenStream {
        let target_name = self.target_name;
        let builder_name = self.builder_name;
        let group_partials = self.generics_gen.builder_const_generic_group_partial_idents();
        let generic_consts = self.generics_gen.builder_const_generic_idents_build();
        let correctness_verifier = self.group_gen.builder_build_impl_correctness_verifier();
        let correctness_check = self.group_gen.builder_build_impl_correctness_check();
        let correctness_helper_fns = self.group_gen.builder_build_impl_correctness_helper_fns();
        let (_, type_generics, where_clause) = self.generics_gen.target_generics().split_for_impl();

        quote!(
            impl #group_partials #builder_name #generic_consts #where_clause{
                #correctness_verifier
                #correctness_helper_fns

                pub fn build(self) -> #target_name #type_generics {
                    #correctness_check
                    self.data.into()
                }
            }
        )
    }

    fn generate_setters_impl(&self) -> StreamResult {
        let builder_name = self.builder_name;
        let setters = self
            .field_gen
            .fields().iter().map(|field| {
                let const_idents_generic = self.generics_gen.builder_const_generic_idents_set_before(field);
                let const_idents_input = self.generics_gen.builder_const_generic_idents_set_after(field, false);
                let const_idents_output = self.generics_gen.builder_const_generic_idents_set_after(field, true);

                let field_name = field.ident();
                let input_type = self.field_gen.builder_set_impl_input_type(field);
                let input_value = self.field_gen.builder_set_impl_input_value(field);
                let where_clause = &self.generics_gen.target_generics().where_clause;

                let tokens = quote!(
                    impl #const_idents_generic #builder_name #const_idents_input #where_clause {
                        pub fn #field_name (self, #input_type) -> #builder_name #const_idents_output {
                            let mut data = self.data;
                            data.#field_name = #input_value;
                            #builder_name {
                                data,
                            }
                        }
                    }
                );
                Ok(tokens)
            }).collect::<VecStreamResult>()?;

        let tokens = quote!(
            #(#setters)*
        );
        Ok(tokens)
    }
}
