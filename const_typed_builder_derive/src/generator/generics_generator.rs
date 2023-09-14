use crate::{info::FieldInfo, MANDATORY_PREFIX};
use either::Either;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

/// The `GenericsGenerator` struct is responsible for generating code related to generics in the target struct, builder, and data types.
#[derive(Debug, Clone)]
pub(super) struct GenericsGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
    target_generics: &'a syn::Generics,
}

/// Creates a new `GenericsGenerator` instance.
///
/// # Arguments
///
/// - `fields`: A reference to a slice of `FieldInfo` representing the fields of the struct.
/// - `target_generics`: A reference to the generics of the target struct.
///
/// # Returns
///
/// A `GenericsGenerator` instance initialized with the provided fields and target generics.
impl<'a> GenericsGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo], target_generics: &'a syn::Generics) -> Self {
        Self {
            fields,
            target_generics,
        }
    }

    /// Returns a reference to the target generics of the struct.
    pub fn target_generics(&self) -> &syn::Generics {
        self.target_generics
    }

    /// Generates const generics with boolean values and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `value`: A boolean value to set for the const generics.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics.
    pub fn const_generics_valued(&self, value: bool) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => Box::new(std::iter::empty())
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(Either::Right(
                syn::LitBool::new(value, mandatory.ident().span()),
            )))
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Grouped(grouped) => {
                Box::new(
                    grouped.group_indices().iter().map(|(_, _)| {
                        Either::Right(syn::LitBool::new(value, grouped.ident().span()))
                    }),
                ) as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>
            }
        });
        self.add_const_generics_valued_for_type(&mut all)
    }

    /// Generates type const generics for the input type of a builder setter method and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `field_info`: A reference to the `FieldInfo` for which the const generics are generated.
    /// - `value`: A boolean value to set for the const generics.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics for the setter method input type.
    pub fn builder_const_generic_idents_set_type(
        &self,
        field_info: &FieldInfo,
        value: bool,
    ) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => Box::new(std::iter::empty())
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Mandatory(_) if field_info == field => Box::new(std::iter::once(
                Either::Right(syn::LitBool::new(value, field_info.ident().span())),
            ))
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(Either::Left(
                format_ident!("{}_{}", MANDATORY_PREFIX, mandatory.mandatory_index()),
            )))
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Grouped(grouped) if field_info == field => Box::new(
                std::iter::repeat(Either::Right(syn::LitBool::new(
                    value,
                    grouped.ident().span(),
                )))
                .take(grouped.group_indices().len()),
            )
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| Either::Left(group.partial_const_ident(*index))),
            )
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
        });
        self.add_const_generics_valued_for_type(&mut all)
    }

    /// Generates impl const generics for the input type of a builder setter method and returns a token stream.
    ///
    /// # Arguments
    ///
    /// - `field_info`: A reference to the `FieldInfo` for which the const generics are generated.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for the setter method input type.
    pub fn builder_const_generic_idents_set_impl(&self, field_info: &FieldInfo) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            _ if field == field_info => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(field) => Box::new(std::iter::once(format_ident!(
                "{}_{}",
                MANDATORY_PREFIX,
                field.mandatory_index()
            ))) as Box<dyn Iterator<Item = syn::Ident>>,
            FieldInfo::Grouped(field) => Box::new(
                field
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics_for_impl(&mut all)
    }

    // Generates const generics for the builder `build` method and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics for the builder `build` method.
    pub fn builder_const_generic_idents_build(&self) -> TokenStream {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => Box::new(std::iter::empty())
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Mandatory(_) => Box::new(std::iter::once(Either::Right(syn::LitBool::new(
                true,
                proc_macro2::Span::call_site(),
            ))))
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| Either::Left(group.partial_const_ident(*index))),
            )
                as Box<dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>>,
        });
        self.add_const_generics_valued_for_type(&mut all)
    }

    /// Generates const generics for builder group partial identifiers and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for builder group partial identifiers.
    pub fn builder_const_generic_group_partial_idents(&self) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics_for_impl(&mut all)
    }

    /// Generates const generics for the builder struct and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for the builder struct.
    pub fn builder_struct_generics(&self) -> syn::Generics {
        let mut all = self.fields.iter().flat_map(|field| match field {
            FieldInfo::Optional(_) => {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = syn::Ident>>
            }
            FieldInfo::Mandatory(mandatory) => Box::new(std::iter::once(format_ident!(
                "M_{}",
                mandatory.mandatory_index()
            )))
                as Box<dyn Iterator<Item = syn::Ident>>,
            FieldInfo::Grouped(grouped) => Box::new(
                grouped
                    .group_indices()
                    .iter()
                    .map(|(group, index)| group.partial_const_ident(*index)),
            ) as Box<dyn Iterator<Item = syn::Ident>>,
        });
        self.add_const_generics_for_impl(&mut all)
    }

    fn add_const_generics_for_impl(
        &self,
        tokens: &mut dyn Iterator<Item = syn::Ident>,
    ) -> syn::Generics {
        let mut res = self.target_generics.clone();

        tokens.for_each(|token| {
            res.params.push(parse_quote!(const #token: bool));
        });
        res
    }

    fn add_const_generics_valued_for_type(
        &self,
        constants: &mut dyn Iterator<Item = Either<syn::Ident, syn::LitBool>>,
    ) -> TokenStream {
        let params = &self.target_generics.params;
        let tokens: Vec<TokenStream> = constants
            .map(|constant| {
                constant
                    .map_either(|iden| iden.to_token_stream(), |lit| lit.to_token_stream())
                    .into_inner()
            })
            .collect();
        if params.is_empty() {
            quote!(<#(#tokens),*>)
        } else {
            let type_generics = params.iter().map(|param| match param {
                syn::GenericParam::Lifetime(lt) => &lt.lifetime.ident,
                syn::GenericParam::Type(ty) => &ty.ident,
                syn::GenericParam::Const(cnst) => &cnst.ident,
            });
            quote!(< #(#type_generics),*, #(#tokens),* >)
        }
    }
}
