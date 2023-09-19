use crate::{info::FieldInfo, info::FieldKind};
use either::Either;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

#[derive(Debug, Clone)]
pub(super) struct GenericsGenerator<'a> {
    pub fields: &'a [FieldInfo<'a>],
    target_generics: &'a syn::Generics,
}

impl<'a> GenericsGenerator<'a> {
    pub fn new(fields: &'a [FieldInfo], target_generics: &'a syn::Generics) -> Self {
        Self {
            fields,
            target_generics,
        }
    }

    pub fn target_generics(&self) -> &syn::Generics {
        self.target_generics
    }

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

    pub fn builder_const_generic_idents_set_impl(&self, field_info: &FieldInfo) -> syn::Generics {
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            _ if field == field_info => None,
            FieldKind::Mandatory | FieldKind::Grouped => Some(field.const_ident()),
        });
        self.add_const_generics_for_impl(&mut all)
    }

    pub fn builder_const_generic_idents_build(&self) -> TokenStream {
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional => None,
            FieldKind::Mandatory => Some(Either::Right(syn::LitBool::new(
                true,
                proc_macro2::Span::call_site(),
            ))),
            FieldKind::Grouped => Some(Either::Left(field.const_ident())),
        });
        self.add_const_generics_valued_for_type(&mut all)
    }

    pub fn builder_const_generic_group_partial_idents(&self) -> syn::Generics {
        let mut all = self.fields.iter().filter_map(|field| match field.kind() {
            FieldKind::Optional | FieldKind::Mandatory => None,
            FieldKind::Grouped => Some(field.const_ident()),
        });
        self.add_const_generics_for_impl(&mut all)
    }

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
