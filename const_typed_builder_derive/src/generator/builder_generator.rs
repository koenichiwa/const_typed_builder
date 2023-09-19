use super::{
    field_generator::FieldGenerator, generics_generator::GenericsGenerator,
    group_generator::GroupGenerator,
};
use crate::{info::SolveType, StreamResult, VecStreamResult};
use proc_macro2::TokenStream;
use quote::quote;

// The `BuilderGenerator` struct is responsible for generating code related to the builder struct,
/// including its definition, implementation of setter methods, `new` method, and `build` method.
pub(super) struct BuilderGenerator<'a> {
    group_gen: GroupGenerator<'a>,
    field_gen: FieldGenerator<'a>,
    generics_gen: GenericsGenerator<'a>,
    target_name: &'a syn::Ident,
    target_vis: &'a syn::Visibility,
    builder_name: &'a syn::Ident,
    data_name: &'a syn::Ident,
    solve_type: SolveType,
}

impl<'a> BuilderGenerator<'a> {
    /// Creates a new `BuilderGenerator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `group_gen`: The `GroupGenerator` responsible for generating group-related code.
    /// - `field_gen`: The `FieldGenerator` responsible for generating field-related code.
    /// - `generics_gen`: The `GenericsGenerator` responsible for generating generics information.
    /// - `target_name`: A reference to the identifier representing the target struct's name.
    /// - `target_vis`: A reference to the visibility of the target struct.
    /// - `builder_name`: A reference to the identifier representing the builder struct's name.
    /// - `data_name`: A reference to the identifier representing the data struct's name.
    ///
    /// # Returns
    ///
    /// A `BuilderGenerator` instance initialized with the provided information.
    pub fn new(
        group_gen: GroupGenerator<'a>,
        field_gen: FieldGenerator<'a>,
        generics_gen: GenericsGenerator<'a>,
        target_name: &'a syn::Ident,
        target_vis: &'a syn::Visibility,
        builder_name: &'a syn::Ident,
        data_name: &'a syn::Ident,
        solve_type: SolveType,
    ) -> Self {
        Self {
            group_gen,
            field_gen,
            generics_gen,
            target_name,
            target_vis,
            builder_name,
            data_name,
            solve_type,
        }
    }

    // Generates the code for the builder struct and its methods and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `StreamResult` representing the generated code for the builder struct and methods.
    pub fn generate(&self) -> StreamResult {
        let builder_struct = self.generate_struct();
        let builder_impl = self.generate_impl()?;
        let tokens = quote!(
            #builder_struct
            #builder_impl
        );
        Ok(tokens)
    }

    /// Generates the code for the builder struct definition.
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

    /// Generates the implementation code for the builder struct's `new`, `build` and setter methods.
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

    /// Generates the code for the `new` method implementation.
    fn generate_new_impl(&self) -> TokenStream {
        let builder_name = self.builder_name;
        let data_name = self.data_name;

        let type_generics = self.generics_gen.const_generics_valued(false);
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

    /// Generates the code for the `build` method implementation.
    fn generate_build_impl(&self) -> TokenStream {
        let builder_name = self.builder_name;
        let target_name = self.target_name;
        let (impl_generics, target_type_generics, where_clause) =
            self.generics_gen.target_generics().split_for_impl();

        match self.solve_type {
            SolveType::BruteForce => {
                let build_impls =
                    self.group_gen
                        .valid_groupident_combinations()
                        .map(|group_indices| {
                            let type_generics = self
                                .generics_gen
                                .builder_const_generic_idents_build(&group_indices);

                            quote!(
                                impl #impl_generics #builder_name #type_generics #where_clause{

                                    pub fn build(self) -> #target_name #target_type_generics {
                                        self.data.into()
                                    }
                                }
                            )
                        });

                quote!(
                    #(#build_impls)*
                )
            }
            SolveType::Compiler => {
                let builder_name = self.builder_name;
                let impl_generics = self
                    .generics_gen
                    .builder_const_generic_group_partial_idents();
                let type_generics = self
                    .generics_gen
                    .builder_const_generic_idents_build_unset_group();

                let correctness_verifier = self.group_gen.builder_build_impl_correctness_verifier();
                let correctness_check = self.group_gen.builder_build_impl_correctness_check();
                let correctness_helper_fns =
                    self.group_gen.builder_build_impl_correctness_helper_fns();

                let target_name = self.target_name;
                let (_, target_type_generics, where_clause) =
                    self.generics_gen.target_generics().split_for_impl();

                quote!(
                    impl #impl_generics #builder_name #type_generics #where_clause{
                        #correctness_verifier
                        #correctness_helper_fns

                        pub fn build(self) -> #target_name #target_type_generics {
                            #correctness_check
                            self.data.into()
                        }
                    }
                )
            }
        }
    }

    /// Generates the code for the setter methods of the builder.
    fn generate_setters_impl(&self) -> StreamResult {
        let builder_name = self.builder_name;
        let setters = self
            .field_gen
            .fields().iter().map(|field| {
                let const_idents_impl = self.generics_gen.builder_const_generic_idents_set_impl(field);
                let const_idents_type_input = self.generics_gen.builder_const_generic_idents_set_type(field, false);
                let const_idents_type_output = self.generics_gen.builder_const_generic_idents_set_type(field, true);
                let where_clause = &self.generics_gen.target_generics().where_clause;

                let field_name = field.ident();
                let input_type = self.field_gen.builder_set_impl_input_type(field);
                let input_value = self.field_gen.builder_set_impl_input_value(field);

                let tokens = quote!(
                    impl #const_idents_impl #builder_name #const_idents_type_input #where_clause {
                        pub fn #field_name (self, #input_type) -> #builder_name #const_idents_type_output {
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
