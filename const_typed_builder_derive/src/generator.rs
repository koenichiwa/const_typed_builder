use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, token::Token};

use crate::{
    builder_info::BuilderInfo,
    data_info::{self, DataInfo},
    field_info::{FieldInfo, FieldSettings},
    struct_info::StructInfo,
    StreamResult,
};

pub struct Generator<'a> {
    struct_info: StructInfo<'a>,
    builder_info: BuilderInfo,
    data_info: DataInfo,
}

impl<'a> Generator<'a> {
    pub fn new(ast: &syn::DeriveInput) -> Result<Generator, syn::Error> {
        let data = match &ast.data {
            syn::Data::Struct(data) => match &data.fields {
                syn::Fields::Named(fields) => {
                    let struct_info = StructInfo::new(ast, fields)?;
                    let builder_info = BuilderInfo::new(ast);
                    let data_info = DataInfo::new(ast);
                    Generator {
                        struct_info,
                        builder_info,
                        data_info,
                    }

                    // let builder_creation = struct_info.builder_creation_impl()?;
                    // let fields = struct_info
                    //     .included_fields()
                    //     .map(|f| struct_info.field_impl(f))
                    //     .collect::<Result<TokenStream, _>>()?;
                    // let required_fields = struct_info
                    //     .included_fields()
                    //     .filter(|f| f.builder_attr.default.is_none())
                    //     .map(|f| struct_info.required_field_impl(f));
                    // let build_method = struct_info.build_method_impl();

                    // quote! {
                    //     #builder_creation
                    //     #fields
                    //     #(#required_fields)*
                    //     #build_method
                    // }
                }
                syn::Fields::Unnamed(_) => {
                    return Err(syn::Error::new(
                        ast.span(),
                        "Builder is not supported for tuple structs",
                    ))
                }
                syn::Fields::Unit => {
                    return Err(syn::Error::new(
                        ast.span(),
                        "Builder is not supported for unit structs",
                    ))
                }
            },
            syn::Data::Enum(_) => {
                return Err(syn::Error::new(
                    ast.span(),
                    "Builder is not supported for enums",
                ))
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    ast.span(),
                    "Builder is not supported for unions",
                ))
            }
        };
        Ok(data)
    }

    pub fn generate(&self) -> StreamResult {
        let target = self.gen_target_impl();
        let data = self.gen_data();
        let builder = self.gen_builder()?;
        Ok(quote!(
            #target

            #builder

            #data
        ))
    }

    fn gen_target_impl(&self) -> TokenStream {
        let StructInfo {
            name: ref target_name,
            ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;
        quote! {
            impl #target_name {
                pub fn builder() -> #builder_name {
                    #builder_name::new()
                }
            }
        }
    }

    fn gen_data(&self) -> TokenStream {
        let __struct = self.gen_data_struct();
        let __impl = self.gen_data_impl();

        quote!(
            #__struct

            #__impl
        )
    }

    fn gen_data_impl(&self) -> TokenStream {
        let DataInfo {
            name: ref data_name,
            ..
        } = self.data_info;

        let StructInfo {
            name: ref struct_name,
            ref field_infos,
            ..
        } = self.struct_info;

        let fields: Vec<_> = field_infos
            .iter()
            .map(|info| {
                let FieldInfo { name: field_name, ty, settings, .. } = info;
                let FieldSettings { mandatory, ..} = settings;

                match (mandatory, info.is_option()) {
                    (true, true) => {
                        quote!(#field_name: data.#field_name)
                    }
                    (true, false) => {
                        quote!(#field_name: data.#field_name.unwrap())
                    }
                    (false, true) => {
                        // Should I collapse or not?
                        quote!(#field_name: data.#field_name)
                    }
                    (false, false) => unreachable!("Non-optional types are always mandatory"),
                }
            })
            .collect();

        quote!(
            impl From<#data_name> for #struct_name {
                fn from(data: #data_name) -> #struct_name {
                    #struct_name {
                        #(#fields),*
                    }
                }
            }
        )
    }

    fn gen_data_struct(&self) -> TokenStream {
        let DataInfo {
            name: ref data_name,
            ..
        } = self.data_info;

        let StructInfo {
            ref field_infos, ..
        } = self.struct_info;

        let fields: Vec<_> = field_infos
            .iter()
            .map(|info| {
                let FieldInfo {
                    name: field_name,
                    ty,
                    settings,
                    ..
                } = info;

                let FieldSettings { mandatory, .. } = settings;

                let comment = format!("mandatory {} option {}", mandatory, info.is_option());

                let field_typed = match (mandatory, info.is_option()) {
                    (true, true) => {
                        quote!(#field_name: #ty)
                    }
                    (true, false) => {
                        quote!(#field_name: Option<#ty>)
                    }
                    (false, true) => {
                        // Should I collapse or not?
                        quote!(#field_name: #ty)
                    }
                    (false, false) => unreachable!("Non-optional types are always mandatory"),
                };

                quote!(
                    #[doc = #comment]
                    pub #field_typed
                )
            })
            .collect();

        quote!(
            #[derive(Default, Debug)]
            pub struct #data_name {
                #(#fields),*
            }
        )
    }

    fn gen_builder(&self) -> StreamResult {
        let __struct = self.gen_builder_struct();
        let __impl = self.gen_builder_impl()?;
        Ok(quote!(
            #__struct

            #__impl
        ))
    }

    fn gen_builder_struct(&self) -> TokenStream {
        let DataInfo {
            name: ref data_name,
            ..
        } = self.data_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name {
                data: #data_name
            }
        )
    }

    fn gen_builder_impl(&self) -> StreamResult {
        let __new = self.gen_builder_new_impl();
        let __setters = self.gen_builder_setters_impl()?;
        let __build = self.gen_builder_build_impl();
        Ok(quote!(
            #__new

            #__setters

            #__build
        ))
    }

    fn gen_builder_new_impl(&self) -> TokenStream {
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;
        quote!(
            impl #builder_name {
                pub fn new() -> #builder_name {
                    Self::default()
                }
            }
        )
    }

    fn gen_builder_build_impl(&self) -> TokenStream {
        let StructInfo {
            name: ref target_name,
            ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;
        quote!(
            impl #builder_name {
                pub fn build(self) -> #target_name {
                    self.data.into()
                }
            }
        )
    }

    fn gen_builder_setters_impl(&self) -> StreamResult {
        let StructInfo {
            ref field_infos, ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let setters = field_infos
            .iter()
            .map(|info| {
                let FieldInfo {
                    name: field_name,
                    ty,
                    ref settings,
                    ..
                } = info;
                let FieldSettings {
                    mandatory,
                    ref input_name,
                } = settings;

                let (input_typed, input_value) = match (mandatory, info.is_option()) {
                    (true, true) => {
                        let inner_ty = info
                            .inner_type()
                            .ok_or(syn::Error::new(Span::call_site(), "Can't reach inner type"))?;
                        (quote!(#input_name: #inner_ty), quote!(Some(#input_name)))
                    }
                    (true, false) => (quote!(#input_name: #ty), quote!(Some(#input_name))),
                    (false, true) => {
                        // Should I collapse or not?
                        (quote!(#input_name: #ty), quote!(#input_name))
                    }
                    (false, false) => unreachable!("Non-optional types are always mandatory"),
                };

                Ok(quote!(
                    impl #builder_name {
                        pub fn #field_name (mut self, #input_typed) -> #builder_name {
                            self.data.#field_name = #input_value;
                            self
                        }
                    }

                ))
            })
            .collect::<Result<Vec<TokenStream>, syn::Error>>()?;
        Ok(quote!(
            #(#setters)*
        ))
    }
}
