use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::{
    builder_info::BuilderInfo,
    context::Context,
    data_info::DataInfo,
    field_info::{FieldInfo, FieldSettings},
    struct_info::StructInfo,
    StreamResult, VecStreamResult,
};

pub struct Generator<'a> {
    struct_info: StructInfo<'a>,
    builder_info: BuilderInfo,
    data_info: DataInfo,
    context: Context,
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
                        context: Context::new(),
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
        let data = self.gen_data()?;
        let builder = self.gen_builder()?;
        let tokens = quote!(
            #target
            #builder
            #data
        );
        Ok(tokens)
    }

    fn gen_target_impl(&self) -> TokenStream {
        let StructInfo {
            name: ref target_name,
            ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let consts: Vec<syn::LitBool> = self
            .struct_info
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

    fn gen_data(&self) -> StreamResult {
        let __struct = self.gen_data_struct()?;
        let __impl = self.gen_data_impl()?;

        let tokens = quote!(
            #__struct
            #__impl
        );

        Ok(tokens)
    }

    fn gen_data_impl(&self) -> StreamResult {
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
                let FieldInfo {
                    name: field_name, ..
                } = info;
                let mandatory_status = info.mandatory_status()?;
                let tokens = match mandatory_status {
                    crate::field_info::MandatoryStatus::Mandatory => {
                        quote!(#field_name: data.#field_name.unwrap())
                    }
                    crate::field_info::MandatoryStatus::MandatoryOption(_) => {
                        quote!(#field_name: data.#field_name)
                    }
                    crate::field_info::MandatoryStatus::Optional(_) => {
                        quote!(#field_name: data.#field_name)
                    }
                };
                Ok(tokens)
            })
            .collect::<VecStreamResult>()?;

        let tokens = quote!(
            impl From<#data_name> for #struct_name {
                fn from(data: #data_name) -> #struct_name {
                    #struct_name {
                        #(#fields),*
                    }
                }
            }
        );
        Ok(tokens)
    }

    fn gen_data_struct(&self) -> StreamResult {
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
                    ..
                } = info;

                // let comment = format!("mandatory {} option {}", mandatory, info.is_option());

                let field_typed = match info.mandatory_status()? {
                    crate::field_info::MandatoryStatus::Mandatory => {
                        quote!(#field_name: Option<#ty>)
                    }
                    crate::field_info::MandatoryStatus::MandatoryOption(_) => {
                        quote!(#field_name: #ty)
                    }
                    crate::field_info::MandatoryStatus::Optional(_) => quote!(#field_name: #ty),
                };

                let tokens = quote!(
                    // #[doc = #comment]
                    pub #field_typed
                );
                Ok(tokens)
            })
            .collect::<VecStreamResult>()?;

        let tokens = quote!(
            #[derive(Default, Debug)]
            pub struct #data_name {
                #(#fields),*
            }
        );
        Ok(tokens)
    }

    fn gen_builder(&self) -> StreamResult {
        let __struct = self.gen_builder_struct();
        let __impl = self.gen_builder_impl()?;
        let tokens = quote!(
            #__struct
            #__impl
        );
        Ok(tokens)
    }

    fn gen_builder_struct(&self) -> TokenStream {
        let DataInfo {
            name: ref data_name,
            ..
        } = self.data_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let const_idents = self.struct_info.mandatory_identifiers();
        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name<#(const #const_idents: bool),*> {
                data: #data_name
            }
        )
    }

    fn gen_builder_impl(&self) -> StreamResult {
        let __new = self.gen_builder_new_impl();
        let __setters = self.gen_builder_setters_impl()?;
        let __build = self.gen_builder_build_impl();
        let tokens = quote!(
            #__new
            #__setters
            #__build
        );
        Ok(tokens)
    }

    fn gen_builder_new_impl(&self) -> TokenStream {
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let consts: Vec<syn::LitBool> = self
            .struct_info
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
        let StructInfo {
            name: ref target_name,
            ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let consts: Vec<syn::LitBool> = self
            .struct_info
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

    fn gen_builder_setters_impl(&self) -> StreamResult {
        let StructInfo {
            ref field_infos, ..
        } = self.struct_info;
        let BuilderInfo {
            name: ref builder_name,
        } = self.builder_info;

        let setters = field_infos
            .iter()
            .map(|field| {
                let FieldInfo {
                    name: field_name,
                    ty,
                    ref settings,
                    ..
                } = field;
                let FieldSettings {
                    ref input_name,
                    ..
                } = settings;

                let const_idents_generic: Vec<_> = self.struct_info.mandatory_identifiers().iter().filter_map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        None
                    } else {
                        Some(ident.clone())
                    }
                }).collect();

                let const_idents_input: Vec<_> = self.struct_info.mandatory_identifiers().iter().map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        quote!(false)
                    } else {
                        quote!(#ident)
                    }
                }).collect();

                let const_idents_output: Vec<_> = self.struct_info.mandatory_identifiers().iter().map(|ident|{
                    if Some(ident) == field.mandatory_ident().as_ref() {
                        quote!(true)
                    } else {
                        quote!(#ident)
                    }
                }).collect();

                let (input_typed, input_value) = match field.mandatory_status()? {
                    crate::field_info::MandatoryStatus::Mandatory => (quote!(#input_name: #ty), quote!(Some(#input_name))),
                    crate::field_info::MandatoryStatus::MandatoryOption(inner_ty) => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
                    crate::field_info::MandatoryStatus::Optional(_) => (quote!(#input_name: #ty), quote!(#input_name)),
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
                Ok(tokens)
            })
            .collect::<VecStreamResult>()?;
        let tokens = quote!(
            #(#setters)*
        );
        Ok(tokens)
    }
}
