use crate::{info::FieldInfo, info::FieldKind};
use either::Either;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
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
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            FieldKind::Mandatory | FieldKind::Grouped => Some(Either::Right(syn::LitBool::new(
                value,
                field.ident().span(),
            ))),
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
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            _ if field == field_info => Some(Either::Right(syn::LitBool::new(
                value,
                field_info.ident().span(),
            ))),
            FieldKind::Mandatory | FieldKind::Grouped => Some(Either::Left(field.const_ident())),
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
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            _ if field == field_info => None,
            FieldKind::Mandatory | FieldKind::Grouped => Some(field.const_ident()),
        });
        self.add_const_generics_for_impl(&mut all)
    }

    // Generates const generics for the builder `build` method and returns a token stream.
    ///
    /// # Returns
    ///
    /// A `TokenStream` representing the generated const generics for the builder `build` method.
    pub fn builder_const_generic_idents_build(&self, except_indices: &[usize]) -> TokenStream {
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            FieldKind::Mandatory => Some(Either::Right(syn::LitBool::new(
                true,
                proc_macro2::Span::call_site(),
            ))),
            FieldKind::Grouped if except_indices.contains(&field.index()) => Some(Either::Right(
                syn::LitBool::new(true, proc_macro2::Span::call_site()),
            )),
            FieldKind::Grouped => Some(Either::Right(syn::LitBool::new(
                false,
                proc_macro2::Span::call_site(),
            ))),
        });
        self.add_const_generics_valued_for_type(&mut all)
    }

    /// Generates const generics for the builder struct and returns a `syn::Generics` instance.
    ///
    /// # Returns
    ///
    /// A `syn::Generics` instance representing the generated const generics for the builder struct.
    pub fn builder_struct_generics(&self) -> syn::Generics {
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            FieldKind::Mandatory | FieldKind::Grouped => Some(field.const_ident()),
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
