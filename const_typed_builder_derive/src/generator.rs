use proc_macro2::TokenStream;
use quote::quote;
use syn::token::Token;

use crate::{
    context::Context,
    field_info::{FieldInfo, FieldSettings},
    struct_info::StructInfo,
    StreamResult, VecStreamResult,
};

pub struct Generator<'a> {
    info: StructInfo<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(info: StructInfo<'a>) -> Self {
        Generator { info }
    }

    pub fn generate(&self, context: &mut Context) -> Option<TokenStream> {
        let target = self.gen_target_impl();
        let data = self.gen_data(context)?;
        let builder = self.gen_builder(context)?;
        let tokens = quote!(
            #target
            #builder
            #data
        );
        Some(tokens)
    }

    fn gen_target_impl(&self) -> TokenStream {
        let target_name = self.info.name();
        let builder_name = self.info.builder_name();

        let consts: Vec<syn::LitBool> = self
            .info
            .mandatory_identifiers()
            .iter()
            .map(|ident| syn::LitBool::new(false, ident.span()))
            .collect();
        quote! {
            impl #target_name {
                pub fn builder() -> #builder_name<#(#consts),*> {
                    #builder_name::new()
                }
            }
        }
    }

    fn gen_data(&self, context: &mut Context) -> Option<TokenStream> {
        let __struct = self.gen_data_struct(context)?;
        let __impl = self.gen_data_impl(context)?;

        let tokens = quote!(
            #__struct
            #__impl
        );

        Some(tokens)
    }

    fn gen_data_impl(&self, context: &mut Context) -> Option<TokenStream> {
        let data_name = self.info.data_name();
        let struct_name = self.info.name();
        let field_infos = self.info.field_infos();

        let fields: Vec<_> = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();
                let mandatory_status = field.type_kind(context)?;
                let tokens = match mandatory_status {
                    crate::field_info::TypeKind::Mandatory { .. } => {
                        quote!(#field_name: data.#field_name.unwrap())
                    }
                    crate::field_info::TypeKind::MandatoryOption { .. } => {
                        quote!(#field_name: data.#field_name)
                    }
                    crate::field_info::TypeKind::Optional { .. } => {
                        quote!(#field_name: data.#field_name)
                    }
                };
                Some(tokens)
            })
            .collect::<Option<Vec<TokenStream>>>()?;

        let tokens = quote!(
            impl From<#data_name> for #struct_name {
                fn from(data: #data_name) -> #struct_name {
                    #struct_name {
                        #(#fields),*
                    }
                }
            }
        );
        Some(tokens)
    }

    fn gen_data_struct(&self, context: &mut Context) -> Option<TokenStream> {
        let field_infos = self.info.field_infos();
        let data_name = self.info.data_name();

        let fields: Vec<_> = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();

                // let comment = format!("mandatory {} option {}", mandatory, info.is_option());

                let field_typed = match field.type_kind(context)? {
                    crate::field_info::TypeKind::Mandatory { ty } => {
                        quote!(#field_name: Option<#ty>)
                    }
                    crate::field_info::TypeKind::MandatoryOption { ty, .. } => {
                        quote!(#field_name: #ty)
                    }
                    crate::field_info::TypeKind::Optional { ty, .. } => quote!(#field_name: #ty),
                };

                let tokens = quote!(
                    // #[doc = #comment]
                    pub #field_typed
                );
                Some(tokens)
            })
            .collect::<Option<Vec<TokenStream>>>()?;

        let tokens = quote!(
            #[derive(Default, Debug)]
            pub struct #data_name {
                #(#fields),*
            }
        );
        Some(tokens)
    }

    fn gen_builder(&self, context: &mut Context) -> Option<TokenStream> {
        let __struct = self.gen_builder_struct();
        let __impl = self.gen_builder_impl(context)?;
        let tokens = quote!(
            #__struct
            #__impl
        );
        Some(tokens)
    }

    fn gen_builder_struct(&self) -> TokenStream {
        let data_name = self.info.data_name();
        let builder_name = self.info.builder_name();
        let const_idents = self.info.mandatory_identifiers();

        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name<#(const #const_idents: bool),*> {
                data: #data_name
            }
        )
    }

    fn gen_builder_impl(&self, context: &mut Context) -> Option<TokenStream> {
        let __new = self.gen_builder_new_impl();
        let __setters = self.gen_builder_setters_impl(context)?;
        let __build = self.gen_builder_build_impl();
        let tokens = quote!(
            #__new
            #__setters
            #__build
        );
        Some(tokens)
    }

    fn gen_builder_new_impl(&self) -> TokenStream {
        let builder_name = self.info.builder_name();

        let consts: Vec<syn::LitBool> = self
            .info
            .mandatory_identifiers()
            .iter()
            .map(|ident| syn::LitBool::new(false, ident.span()))
            .collect();

        quote!(
            impl #builder_name<#(#consts),*> {
                pub fn new() -> #builder_name<#(#consts),*> {
                    Self::default()
                }
            }
        )
    }

    fn gen_builder_build_impl(&self) -> TokenStream {
        let target_name = self.info.name();
        let builder_name = self.info.builder_name();

        let consts: Vec<syn::LitBool> = self
            .info
            .mandatory_identifiers()
            .iter()
            .map(|ident| syn::LitBool::new(true, ident.span()))
            .collect();

        quote!(
            impl #builder_name<#(#consts),*> {
                pub fn build(self) -> #target_name {
                    self.data.into()
                }
            }
        )
    }

    fn gen_builder_setters_impl(&self, context: &mut Context) -> Option<TokenStream> {
        let field_infos = self.info.field_infos();
        let builder_name = self.info.builder_name();

        let setters = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();
                let input_name = field.input_name();

                let const_idents_generic: Vec<_> = self.info.mandatory_identifiers().iter().filter_map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        None
                    } else {
                        Some(ident.clone())
                    }
                }).collect();

                let const_idents_input: Vec<_> = self.info.mandatory_identifiers().iter().map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        quote!(false)
                    } else {
                        quote!(#ident)
                    }
                }).collect();

                let const_idents_output: Vec<_> = self.info.mandatory_identifiers().iter().map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        quote!(true)
                    } else {
                        quote!(#ident)
                    }
                }).collect();

                let (input_typed, input_value) = match field.type_kind(context)? {
                    crate::field_info::TypeKind::Mandatory { ty } => (quote!(#input_name: #ty), quote!(Some(#input_name))),
                    crate::field_info::TypeKind::MandatoryOption {ty, inner_ty } => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
                    crate::field_info::TypeKind::Optional {ty, inner_ty } => (quote!(#input_name: #ty), quote!(#input_name)),
                };

                let tokens = quote!(
                    impl <#(const #const_idents_generic: bool),*> #builder_name <#(#const_idents_input),*> {
                        pub fn #field_name (self, #input_typed) -> #builder_name <#(#const_idents_output),*> {
                            let mut data = self.data;
                            data.#field_name = #input_value;
                            #builder_name {
                                data,
                            }
                        }
                    }
                );
                Some(tokens)
            })
            .collect::<Option<Vec<TokenStream>>>()?;
        let tokens = quote!(
            #(#setters)*
        );
        Some(tokens)
    }
}
