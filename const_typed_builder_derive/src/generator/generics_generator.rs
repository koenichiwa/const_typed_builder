use crate::{
    info::FieldInfo,
    info::{FieldInfoCollection, FieldKind},
};
use either::Either;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

/// The `GenericsGenerator` struct is responsible for generating code related to generics in the target struct, builder, and data types.
#[derive(Debug, Clone)]
pub(super) struct GenericsGenerator<'a> {
    pub field_infos: &'a FieldInfoCollection<'a>,
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
    pub fn new(fields: &'a FieldInfoCollection, target_generics: &'a syn::Generics) -> Self {
        Self {
            field_infos: fields,
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
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional => None,
                FieldKind::Mandatory | FieldKind::Grouped => {
                    Some(syn::LitBool::new(value, field.ident().span()).to_token_stream())
                }
            });

        if let FieldInfoCollection::EnumFields { .. } = self.field_infos {
            self.add_const_generics_valued_for_type(&mut std::iter::once(quote!(0usize)).chain(all))
        } else {
            self.add_const_generics_valued_for_type(&mut all)
        }
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
        match self.field_infos {
            FieldInfoCollection::StructFields { fields } => {
                let mut all = fields.iter().filter_map(|field| match field.kind() {
                    FieldKind::Skipped | FieldKind::Optional => None,
                    _ if field == field_info => {
                        Some(syn::LitBool::new(value, field_info.ident().span()).to_token_stream())
                    }
                    FieldKind::Mandatory | FieldKind::Grouped => {
                        Some(field.const_ident().to_token_stream())
                    }
                });
                self.add_const_generics_valued_for_type(&mut all)
            }
            FieldInfoCollection::EnumFields { variant_fields } => {
                let (variant_index, fields_infos) = variant_fields.iter().enumerate().fold(
                    (0, Vec::new()),
                    |(mut variant_index, mut my_iter), (index, (variant, fields))| {
                        let new_iter = fields.iter().filter_map(|field| match field.kind() {
                            FieldKind::Skipped | FieldKind::Optional => None,
                            _ if field == field_info => {
                                variant_index = index + 1;
                                Some(
                                    syn::LitBool::new(value, field_info.ident().span())
                                        .to_token_stream(),
                                )
                            }
                            FieldKind::Mandatory | FieldKind::Grouped => {
                                Some(field.const_ident().to_token_stream())
                            }
                        });
                        my_iter.extend(new_iter);
                        (variant_index, my_iter)
                    },
                );
                self.add_const_generics_valued_for_type(
                    &mut std::iter::once(quote!(#variant_index)).chain(fields_infos),
                )
            }
        }
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
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional => None,
                _ if field == field_info => None,
                FieldKind::Mandatory | FieldKind::Grouped => Some(field.const_ident()),
            });
        self.add_const_generics_for_impl(&mut all)
    }

    /// Generates const generics for the builder `build` method and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics for the builder `build` method.
    pub fn builder_const_generic_idents_build(&self, true_indices: &[usize]) -> TokenStream {
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional => None,
                FieldKind::Mandatory => {
                    Some(syn::LitBool::new(true, proc_macro2::Span::call_site()).to_token_stream())
                }
                FieldKind::Grouped if true_indices.contains(&field.index()) => {
                    Some(syn::LitBool::new(true, proc_macro2::Span::call_site()).to_token_stream())
                }
                FieldKind::Grouped => {
                    Some(syn::LitBool::new(false, proc_macro2::Span::call_site()).to_token_stream())
                }
            });
        self.add_const_generics_valued_for_type(&mut all)
    }

    // Generates const generics for the builder `build` method and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics for the builder `build` method.
    pub fn builder_const_generic_idents_build_unset_group(&self) -> TokenStream {
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional => None,
                FieldKind::Mandatory => Some(
                    Box::new(syn::LitBool::new(true, proc_macro2::Span::call_site()))
                        .to_token_stream(),
                ),
                FieldKind::Grouped => Some(field.const_ident().to_token_stream()),
            });
        self.add_const_generics_valued_for_type(&mut all)
    }

    /// Generates const generics for builder group partial identifiers and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for builder group partial identifiers.
    pub fn builder_const_generic_group_partial_idents(&self) -> syn::Generics {
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional | FieldKind::Mandatory => None,
                FieldKind::Grouped => Some(field.const_ident()),
            });
        let mut res = self.add_const_generics_for_impl(&mut all);
        if let FieldInfoCollection::EnumFields { .. } = self.field_infos {
            res.params
                .push(parse_quote!(const __BUILDER_CONST_VARIANT: usize));
            res
        } else {
            res
        }
    }

    /// Generates const generics for the builder struct and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for the builder struct.
    pub fn builder_struct_generics(&self) -> syn::Generics {
        let mut all = self
            .field_infos
            .all_fields()
            .into_iter()
            .filter_map(|field| match field.kind() {
                FieldKind::Skipped | FieldKind::Optional => None,
                FieldKind::Mandatory | FieldKind::Grouped => Some(field.const_ident()),
            });
        let mut res = self.add_const_generics_for_impl(&mut all);
        if let FieldInfoCollection::EnumFields { .. } = self.field_infos {
            res.params
                .insert(0, parse_quote!(const __BUILDER_CONST_VARIANT: usize));
        }
        res
    }

    /// Adds const generic identifiers to the target structs `syn::Generics` and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generics for the builder struct.
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
    /// Adds valued const generics to the target structs `syn::Generics` and returns a `Tokenstream` instance.
    ///
    /// # Returns
    ///
    /// A `Tokenstream` instance representing the generics for the builder struct.
    fn add_const_generics_valued_for_type(
        &self,
        constants: &mut dyn Iterator<Item = TokenStream>,
    ) -> TokenStream {
        let params = &self.target_generics.params;
        if params.is_empty() {
            quote!(<#(#constants),*>)
        } else {
            let type_generics = params.iter().map(|param| match param {
                syn::GenericParam::Lifetime(lt) => &lt.lifetime.ident,
                syn::GenericParam::Type(ty) => &ty.ident,
                syn::GenericParam::Const(cnst) => &cnst.ident,
            });
            quote!(< #(#type_generics),*, #(#constants),* >)
        }
    }
}
