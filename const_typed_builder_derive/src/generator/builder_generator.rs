use super::util;
use crate::info::{
    Container, Field, FieldKind, GroupType, SetterKind, SolverKind, TrackedField, TrackedFieldKind,
};
use itertools::{Itertools, Powerset};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use std::{collections::BTreeSet, ops::Deref};
use syn::{parse_quote, GenericParam};

pub struct BuilderGenerator<'info> {
    info: &'info Container<'info>,
}

impl<'info> BuilderGenerator<'info> {
    /// Creates a new `BuilderGenerator` instance for code generation.
    ///
    /// # Arguments
    ///
    /// - `info`: The `Container` containing all the information of the data container.
    ///
    /// # Returns
    ///
    /// A `BuilderGenerator` instance initialized with the provided information.
    pub fn new(info: &'info Container) -> Self {
        Self { info }
    }

    // Generates the code for the builder struct and its methods and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `Tokenstream` representing the generated code for the builder struct and methods.
    pub fn generate(&self) -> TokenStream {
        let builder_struct = self.generate_struct();
        let builder_impl = self.generate_impl();
        quote!(
            #builder_struct
            #builder_impl
        )
    }

    /// Generates the code for the builder struct definition.
    fn generate_struct(&self) -> TokenStream {
        let data_ident = self.info.data_ident();
        let data_field = self.info.data_field_ident();
        let builder_ident = self.info.builder_ident();

        let generics = self.struct_generics();
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let (_, type_generics, _) = self.info.generics().split_for_impl();

        let vis = self.info.vis();

        let documentation = format!(
            "Builder for [`{}`] derived using the `const_typed_builder` crate",
            self.info.ident()
        );

        quote!(
            #[doc = #documentation]
            #vis struct #builder_ident #impl_generics #where_clause {
                #data_field: #data_ident #type_generics
            }
        )
    }

    /// Generates the implementation code for the builder struct's `new`, `build` and setter methods.
    fn generate_impl(&self) -> TokenStream {
        let builder_setters = self.generate_setters_impl();
        let builder_new = self.generate_new_impl();
        let builder_build = self.generate_build_impl();

        quote!(
            #builder_new
            #builder_setters
            #builder_build
        )
    }

    /// Generates the code for the `new` method implementation.
    fn generate_new_impl(&self) -> TokenStream {
        let builder_ident = self.info.builder_ident();
        let data_ident = self.info.data_ident();
        let data_field = self.info.data_field_ident();

        let type_generics = util::const_generics_all_valued(
            false,
            self.info.field_collection(),
            self.info.generics(),
        );
        let (impl_generics, _, where_clause) = self.info.generics().split_for_impl();
        let documentation =
            format!("Creates a new [`{builder_ident}`] without any fields initialized");

        quote!(
            impl #impl_generics #builder_ident #type_generics #where_clause{
                #[doc = #documentation]
                pub fn new() -> #builder_ident #type_generics {
                    Self::default()
                }
            }

            impl #impl_generics Default for #builder_ident #type_generics #where_clause {
                #[doc = #documentation]
                fn default() -> Self {
                    #builder_ident {
                        #data_field: #data_ident::default(),
                    }
                }
            }
        )
    }

    /// Generates the code for the `build` method implementation.
    fn generate_build_impl(&self) -> TokenStream {
        let builder_ident = self.info.builder_ident();
        let target_ident = self.info.ident();
        let data_field = self.info.data_field_ident();
        let documentation =
            format!("Build an instance of [`{target_ident}`], consuming the [`{builder_ident}`]");

        let (impl_generics, target_type_generics, where_clause) =
            self.info.generics().split_for_impl();

        match self.info.solver_kind() {
            SolverKind::BruteForce => {
                let build_impls = self.valid_groupident_combinations().map(|group_indices| {
                    let type_generics = self.const_generic_idents_build(&group_indices);

                    quote!(
                        impl #impl_generics #builder_ident #type_generics #where_clause{
                            #[doc = #documentation]
                            pub fn build(self) -> #target_ident #target_type_generics {
                                self.#data_field.into()
                            }
                        }
                    )
                });

                quote!(
                    #(#build_impls)*
                )
            }
            SolverKind::Compiler => {
                let builder_ident = self.info.builder_ident();
                let impl_generics = self.const_generic_group_partial_idents();
                let type_generics = self.const_generic_idents_build_unset_group();

                let correctness_verifier = self.impl_correctness_verifier();
                let correctness_check = self.impl_correctness_check();
                let correctness_helper_fns = self.impl_correctness_helper_fns();

                let target_ident = self.info.ident();
                let (_, target_type_generics, where_clause) = self.info.generics().split_for_impl();

                quote!(
                    impl #impl_generics #builder_ident #type_generics #where_clause{
                        #correctness_verifier
                        #correctness_helper_fns

                        #[doc = #documentation]
                        pub fn build(self) -> #target_ident #target_type_generics {
                            #correctness_check
                            self.#data_field.into()
                        }
                    }
                )
            }
        }
    }

    /// Generates the code for the setter methods of the builder.
    fn generate_setters_impl(&self) -> TokenStream {
        let builder_ident = self.info.builder_ident();
        let data_field = self.info.data_field_ident();
        let setters = self
            .info
            .field_collection()
            .iter()
            .filter(|field| field.kind() != FieldKind::Skipped)
            .map(|field| {
                let const_idents_impl = self.const_generic_idents_set_impl(field);
                let const_idents_type_input = self.const_generic_idents_set_type(field, false);
                let const_idents_type_output = self.const_generic_idents_set_type(field, true);
                let where_clause = &self.info.generics().where_clause;

                let field_ident = field.ident();
                let input_type = self.field_input_type(field);
                let input_value = self.field_input_assign(field);

                let documentation = format!(r#"
Setter for the [`{}::{field_ident}`] field.

# Arguments

- `{field_ident}`: field to be set

# Returns

`{builder_ident}` with `{field_ident}` initialized"#, self.info.ident());

                quote!(
                    impl #const_idents_impl #builder_ident #const_idents_type_input #where_clause {
                        #[doc = #documentation]
                        pub fn #field_ident (mut self, #field_ident: #input_type) -> #builder_ident #const_idents_type_output {
                            self.#data_field.#field_ident = #input_value;
                            #builder_ident {
                                #data_field: self.#data_field,
                            }
                        }
                    }
                )
            });

        quote!(
            #(#setters)*
        )
    }

    fn struct_generics(&self) -> syn::Generics {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(TrackedField::new)
            .map(|field| field.const_ident());
        self.add_const_generics_for_impl(&mut all)
    }

    fn const_generic_idents_build(&self, true_indices: &[usize]) -> TokenStream {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(TrackedField::new)
            .map(|field| match field.kind() {
                TrackedFieldKind::Mandatory => quote!(true),
                TrackedFieldKind::Grouped if true_indices.contains(&field.index()) => {
                    quote!(true)
                }
                TrackedFieldKind::Grouped => quote!(false),
            });
        util::add_const_valued_generics_for_type(&mut all, self.info.generics())
    }

    fn const_generic_idents_set_impl(&self, field_info: &Field) -> syn::Generics {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(TrackedField::new)
            .filter_map(|field| {
                if field.deref() == field_info {
                    None
                } else {
                    Some(field.const_ident())
                }
            });
        self.add_const_generics_for_impl(&mut all)
    }

    fn const_generic_idents_set_type(&self, field_info: &Field, value: bool) -> TokenStream {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(TrackedField::new)
            .map(|field| {
                if field.deref() == field_info {
                    quote!(#value)
                } else {
                    let ident = field.const_ident();
                    quote!(#ident)
                }
            });
        util::add_const_valued_generics_for_type(&mut all, self.info.generics())
    }

    fn const_generic_group_partial_idents(&self) -> syn::Generics {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Grouped => Some(field.const_ident()),
                FieldKind::Optional | FieldKind::Skipped | FieldKind::Mandatory => None,
            });
        self.add_const_generics_for_impl(&mut all)
    }

    fn const_generic_idents_build_unset_group(&self) -> TokenStream {
        let mut all = self
            .info
            .field_collection()
            .iter()
            .filter_map(TrackedField::new)
            .map(|field| match field.kind() {
                TrackedFieldKind::Mandatory => quote!(true),
                TrackedFieldKind::Grouped => {
                    let ident = field.const_ident();
                    quote!(#ident)
                }
            });
        util::add_const_valued_generics_for_type(&mut all, self.info.generics())
    }

    fn impl_correctness_verifier(&self) -> TokenStream {
        if self.info.group_collection().is_empty() {
            return TokenStream::new();
        }

        let all = self.info.group_collection().values().map(|group| {
            let partials = group.indices().iter().map(|index| self.info.field_collection().get(*index).expect("Could not find field associated to group").const_ident());
            let function_call = syn::Ident::new(group.function_symbol().as_ref(), Span::call_site());
            let count = group.expected_count();
            let ident = group.ident();
            let function_name = group.function_symbol();
            let err_text = format!("`.build()` failed because the bounds of group `{ident}` where not met ({function_name} {count})");

            quote!(
                if !Self::#function_call(&[#(#partials),*], #count) {
                    panic!(#err_text);
                }
            )
        });
        quote!(
            const GROUP_VERIFIER: ()  = {
                #(#all)*
                ()
            };
        )
    }

    fn impl_correctness_check(&self) -> TokenStream {
        if self.info.group_collection().is_empty() {
            TokenStream::new()
        } else {
            quote!(let _ = Self::GROUP_VERIFIER;)
        }
    }

    fn impl_correctness_helper_fns(&self) -> TokenStream {
        if self.info.group_collection().is_empty() {
            return TokenStream::new();
        }

        let mut exact = false;
        let mut at_least = false;
        let mut at_most = false;

        for group in self.info.group_collection().values() {
            match group.group_type() {
                GroupType::Exact(_) => exact = true,
                GroupType::AtLeast(_) => at_least = true,
                GroupType::AtMost(_) => at_most = true,
            }

            if exact && at_least && at_most {
                break;
            }
        }

        let exact = exact.then(|| {
            quote!(
                const fn exact(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count == count
                }
            )
        });

        let at_least = at_least.then(|| {
            quote!(
                const fn at_least(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count >= count
                }
            )
        });

        let at_most = at_most.then(|| {
            quote!(
                const fn at_most(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len() {
                        if input[i] {
                            this_count += 1
                        }
                        i += 1;
                    }
                    this_count <= count
                }
            )
        });
        quote!(
            #exact
            #at_least
            #at_most
        )
    }

    /// Retrieves the input type for the builder's setter method.
    pub fn field_input_type(&self, field: &Field) -> Option<TokenStream> {
        if field.kind() == FieldKind::Skipped {
            return None;
        }

        let input_type = match field.setter_kind() {
            SetterKind::Standard => match field.kind() {
                FieldKind::Grouped | FieldKind::Mandatory if field.is_option_type() => field
                    .inner_type()
                    .expect("Option types have an inner type")
                    .to_token_stream(),
                FieldKind::Skipped => unreachable!("Skipped fields have an early return"),
                _ => field.ty().to_token_stream(),
            },
            SetterKind::Propagate => {
                let input = if let Some(inner_ty) = field.inner_type() {
                    inner_ty
                } else {
                    field.ty()
                };
                let output = match field.kind() {
                    FieldKind::Grouped | FieldKind::Mandatory if field.is_option_type() => {
                        field.inner_type().expect("Option types have an inner type")
                    }
                    FieldKind::Skipped => unreachable!("Skipped fields have an early return"),
                    _ => field.ty(),
                };
                quote!(fn(<#input as Builder>:: BuilderImpl) -> #output)
            }
            SetterKind::Into => {
                let ty = if let Some(inner_ty) = field.inner_type() {
                    inner_ty
                } else {
                    field.ty()
                };
                if field.kind() == FieldKind::Optional {
                    quote!(Option<impl Into<#ty>>)
                } else {
                    quote!(impl Into<#ty>)
                }
            }
            SetterKind::AsMut => {
                let ty = if let Some(inner_ty) = field.inner_type() {
                    inner_ty
                } else {
                    field.ty()
                };
                if let syn::Type::Reference(syn::TypeReference {
                    lifetime,
                    mutability,
                    elem,
                    ..
                }) = ty
                {
                    if mutability.is_none() {
                        emit_error!(
                            ty,
                            "You need a mutable reference to use this type of setter"
                        );
                    }
                    if field.kind() == FieldKind::Optional {
                        quote!(Option<&#lifetime mut impl AsMut<#elem>>)
                    } else {
                        quote!(&#lifetime mut impl AsMut<#elem>)
                    }
                } else {
                    emit_error!(
                        ty,
                        "You need a mutable reference to use this type of setter"
                    );
                    return None;
                }
            }
            SetterKind::AsRef => {
                let ty = if let Some(inner_ty) = field.inner_type() {
                    inner_ty
                } else {
                    field.ty()
                };
                if let syn::Type::Reference(syn::TypeReference {
                    lifetime,
                    mutability: _,
                    elem,
                    ..
                }) = ty
                {
                    if field.kind() == FieldKind::Optional {
                        quote!(Option<&#lifetime impl AsRef<#elem>>)
                    } else {
                        quote!(&#lifetime impl AsRef<#elem>)
                    }
                } else {
                    emit_error!(ty, "You need a reference to use this type of setter");
                    return None;
                }
            }
        };
        Some(input_type)
    }

    pub fn field_input_assign(&self, field: &Field) -> Option<TokenStream> {
        if field.kind() == FieldKind::Skipped {
            return None;
        }

        let field_ident = field.ident();

        let field_value = match field.setter_kind() {
            SetterKind::Standard => {
                if field.kind() == FieldKind::Optional {
                    quote!(#field_ident)
                } else {
                    quote!(Some(#field_ident))
                }
            }
            SetterKind::Propagate => {
                if let Some(inner_ty) = field.inner_type() {
                    quote!(#field_ident(<#inner_ty as Builder>::builder()))
                } else {
                    let ty = field.ty();
                    quote!(Some(#field_ident(<#ty as Builder>::builder())))
                }
            }
            SetterKind::Into => {
                if field.kind() == FieldKind::Optional {
                    quote!(#field_ident.map(Into::into))
                } else {
                    quote!(Some(#field_ident.into()))
                }
            }
            SetterKind::AsMut => {
                if field.kind() == FieldKind::Optional {
                    quote!(#field_ident.map(AsMut::as_mut))
                } else {
                    quote!(Some(#field_ident.as_mut()))
                }
            }
            SetterKind::AsRef => {
                if field.kind() == FieldKind::Optional {
                    quote!(#field_ident.map(AsRef::as_ref))
                } else {
                    quote!(Some(#field_ident.as_ref()))
                }
            }
        };

        Some(quote!(#field_value))
    }

    fn valid_groupident_combinations(&self) -> impl Iterator<Item = Vec<usize>> + '_ {
        let group_indices: BTreeSet<usize> = self
            .info
            .group_collection()
            .values()
            .flat_map(|group| group.indices().clone())
            .collect();
        let powerset: Powerset<std::collections::btree_set::IntoIter<usize>> =
            group_indices.into_iter().powerset();
        powerset.filter_map(|set| {
            if self
                .info
                .group_collection()
                .values()
                .all(|group| group.is_valid_with(&set))
            {
                Some(set)
            } else {
                None
            }
        })
    }

    /// Adds const generic identifiers to the target structs `syn::Generics` and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generics for the builder struct.
    fn add_const_generics_for_impl(
        &self,
        tokens: &mut impl Iterator<Item = syn::Ident>,
    ) -> syn::Generics {
        let mut res = self.info.generics().clone();
        res.params
            .extend(tokens.map::<GenericParam, _>(|token| parse_quote!(const #token: bool)));
        res
    }
}
