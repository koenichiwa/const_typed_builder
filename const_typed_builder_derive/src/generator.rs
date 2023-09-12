use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{token::{Token, Impl}, Field};

use crate::{
    context::Context,
    field_info::{FieldInfo, FieldSettings, self},
    struct_info::StructInfo,
    StreamResult, VecStreamResult, MANDATORY_PREFIX,
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

        let consts = self.gen_builder_const_generics_valued(false);
        quote! {
            impl #target_name {
                pub fn builder() -> #builder_name #consts {
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
                    crate::field_info::TypeKind::GroupOption { .. } => {
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
                    crate::field_info::TypeKind::Optional { ty, .. } => {
                        quote!(#field_name: #ty)
                    }
                    crate::field_info::TypeKind::GroupOption { ty, inner_ty } => {
                        quote!(#field_name: #ty)
                    }
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
        let funcs = self.gen_builder_group_const_fn();
        let tokens = quote!(
            #__struct
            #__impl
            #funcs
        );
        Some(tokens)
    }

    fn gen_builder_struct(&self) -> TokenStream {
        let data_name = self.info.data_name();
        let builder_name = self.info.builder_name();
        let const_idents = self.gen_builder_const_generic_idents();

        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name #const_idents {
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

        let consts = self.gen_builder_const_generics_valued(false);

        quote!(
            impl #builder_name #consts {
                pub fn new() -> #builder_name #consts {
                    Self::default()
                }
            }
        )
    }

    fn gen_builder_build_impl(&self) -> TokenStream {
        let target_name = self.info.name();
        let builder_name = self.info.builder_name();

        let group_partials = self.gen_builder_const_generic_group_partial_idents();
        let builder_generics = self.gen_builder_const_generic_idents_final();
        let correctness_verifier = self.gen_builder_group_correctness_verifier();

        quote!(
            impl #group_partials #builder_name #builder_generics {

                #correctness_verifier

                pub fn build(self) -> #target_name {
                    let _ = Self::GROUP_VERIFIER;
                    self.data.into()
                }

                const fn exact(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len(){
                        if input[i] { this_count += 1 }
                        i += 1;
                    }
                    this_count == count
                }

                const fn at_least(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len(){
                        if input[i] { this_count += 1 }
                        i += 1;
                    }
                    this_count >= count
                }

                const fn at_most(input: &[bool], count: usize) -> bool {
                    let mut this_count = 0;
                    let mut i = 0;
                    while i < input.len(){
                        if input[i] { this_count += 1 }
                        i += 1;
                    }
                    this_count <= count
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

                let const_idents_generic = self.gen_builder_const_generic_idents_set_before(field);
                let const_idents_input = self.gen_builder_const_generic_idents_set(field, false);
                let const_idents_output = self.gen_builder_const_generic_idents_set(field, true);

                let (input_typed, input_value) = match field.type_kind(context)? {
                    crate::field_info::TypeKind::Mandatory { ty } => (quote!(#input_name: #ty), quote!(Some(#input_name))),
                    crate::field_info::TypeKind::MandatoryOption {ty, inner_ty } => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
                    crate::field_info::TypeKind::Optional {ty, inner_ty } => (quote!(#input_name: #ty), quote!(#input_name)),
                    crate::field_info::TypeKind::GroupOption { ty, inner_ty } => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
                };

                let tokens = quote!(
                    impl #const_idents_generic #builder_name #const_idents_input {
                        pub fn #field_name (self, #input_typed) -> #builder_name #const_idents_output {
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

    fn gen_builder_group_const_fn(&self) -> TokenStream {
        let builder_name = self.info.builder_name();
        let partial_idents = self.gen_builder_const_generic_group_partial_idents();
        let builder_generics = self.gen_builder_const_generic_idents_final();
        quote!(
            impl #partial_idents #builder_name #builder_generics {
                
            }
        )
    }

    fn gen_builder_const_generic_idents(&self) -> TokenStream {
        let mandatory = (0..self.info.mandatory_count()).map(|index| format_ident!("{}_{}", MANDATORY_PREFIX, index));
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .map(|index| group.partial_const_ident(index))
                // .chain(std::iter::once(group.const_ident()))
        });
        let all = mandatory.chain(groups);
        quote!(<#(const #all: bool),*>)
    }

    fn gen_builder_const_generics_valued(&self, value: bool) -> TokenStream {
        let iter = std::iter::repeat(syn::LitBool::new(value, proc_macro2::Span::call_site()).to_token_stream());
        let mandatory = iter.clone().take(self.info.mandatory_count());
        let groups = self.info.groups().values().flat_map(|group| {
            let variables = iter.clone().take(group.member_count());
            let function_name = group.function_ident();
            let function_vars = variables.clone();
            let number = group.expected_count();
            let modname = format_ident!("{}Mod", self.info.name());
            let function_call = quote!({#modname::#function_name(&[#(#function_vars),*], #number)});
            // variables.chain(std::iter::once(function_call))
            variables
        });
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn gen_builder_const_generic_idents_set(&self, field_info: &FieldInfo, value: bool) -> TokenStream {
        let mandatory =  (0..self.info.mandatory_count()).map(|index| {
            if field_info.mandatory_index() == Some(index) {
                syn::LitBool::new(value, proc_macro2::Span::call_site()).into_token_stream()
            } else {
                format_ident!("{}_{}", MANDATORY_PREFIX, index).into_token_stream()
            }
        });
        let groups = self.info.groups().values().flat_map(|group| {
            let variables = (0..group.member_count())
                .map(|index| {
                    if field_info.get_group_index(group) == Some(index) {
                        syn::LitBool::new(value, proc_macro2::Span::call_site()).into_token_stream()
                    } else {
                        group.partial_const_ident(index).into_token_stream()
                    }
                });
            let function_name = group.function_ident();
            let function_vars = variables.clone();
            let number = group.expected_count();
            let modname = format_ident!("{}Mod", self.info.name());
            let function_call = quote!({#modname::#function_name(&[#(#function_vars),*], #number)});
            // variables.chain(std::iter::once(function_call))
            variables
        });
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn gen_builder_const_generic_idents_set_before(&self, field_info: &FieldInfo) -> TokenStream {
        let mandatory =  (0..self.info.mandatory_count()).filter_map(|index| {
            if field_info.mandatory_index() == Some(index) {
                None
            } else {
                Some(format_ident!("{}_{}", MANDATORY_PREFIX, index).into_token_stream())
            }
        });
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .filter_map(|index| {
                    if field_info.get_group_index(group) == Some(index) {
                        None
                    } else {
                        Some(group.partial_const_ident(index).into_token_stream())
                    }
                })
        });
        let all = mandatory.chain(groups);
        quote!(<#(const #all: bool),*>)
    }

    fn gen_builder_const_generic_idents_final(&self) -> TokenStream {
        let value = syn::LitBool::new(true, proc_macro2::Span::call_site()).to_token_stream();
        let iter = std::iter::repeat(value.clone());
        let mandatory = iter.take(self.info.mandatory_count());
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .map(|index| group.partial_const_ident(index).to_token_stream())
                // .chain(std::iter::once(value.clone()))
        });
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn gen_builder_const_generic_group_partial_idents(&self) -> TokenStream {
        let all = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream())
        });
        quote!(<#(const #all: bool),*>)
    }

    fn gen_builder_group_correctness_verifier(&self) -> TokenStream {
        let all = self.info.groups().values().flat_map(|group| {
            let partials = (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream());
            let function_call = group.function_ident();
            let count = group.expected_count();
            let name = group.name();
            quote!(
                if !Self::#function_call(&[#(#partials),*], #count) {
                    panic!("Group #name not verified");
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
}
