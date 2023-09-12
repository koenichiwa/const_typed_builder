use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    field_info::{FieldInfo, TypeKind},
    struct_info::StructInfo,
    StreamResult, MANDATORY_PREFIX,
};

pub struct Generator<'a> {
    info: StructInfo<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(info: StructInfo<'a>) -> Self {
        Generator { info }
    }

    pub fn generate(&self) -> StreamResult {
        let target = self.generate_target_impl();
        let data = self.generate_data()?;
        let builder = self.generate_builder()?;
        let tokens = quote!(
            #target
            #builder
            #data
        );
        Ok(tokens)
    }

    fn generate_target_impl(&self) -> TokenStream {
        let target_name = self.info.name();
        let builder_name = self.info.builder_name();
        let consts = self.generate_builder_const_generics_valued(false);

        quote! {
            impl #target_name {
                pub fn builder() -> #builder_name #consts {
                    #builder_name::new()
                }
            }
        }
    }

    fn generate_data(&self) -> StreamResult {
        let data_struct = self.generate_data_struct()?;
        let data_impl = self.generate_data_impl()?;

        let tokens = quote!(
            #data_struct
            #data_impl
        );

        Ok(tokens)
    }

    fn generate_data_impl(&self) -> StreamResult {
        let data_name = self.info.data_name();
        let struct_name = self.info.name();
        let field_infos = self.info.field_infos();

        let fields: Vec<_> = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();
                let tokens = match field.type_kind()? {
                    TypeKind::Mandatory { .. } => {
                        quote!(#field_name: data.#field_name.unwrap())
                    }
                    TypeKind::MandatoryOption { .. } => {
                        quote!(#field_name: data.#field_name)
                    }
                    TypeKind::Optional { .. } => {
                        quote!(#field_name: data.#field_name)
                    }
                    TypeKind::GroupOption { .. } => {
                        quote!(#field_name: data.#field_name)
                    }
                };
                Ok(tokens)
            })
            .collect::<syn::Result<Vec<TokenStream>>>()?;

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

    fn generate_data_struct(&self) -> StreamResult {
        let field_infos = self.info.field_infos();
        let data_name = self.info.data_name();

        let fields: Vec<_> = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();
                let field_typed = match field.type_kind()? {
                    TypeKind::Mandatory { ty } => {
                        quote!(#field_name: Option<#ty>)
                    }
                    TypeKind::MandatoryOption { ty, .. } => {
                        quote!(#field_name: #ty)
                    }
                    TypeKind::Optional { ty, .. } => {
                        quote!(#field_name: #ty)
                    }
                    TypeKind::GroupOption { ty, .. } => {
                        quote!(#field_name: #ty)
                    }
                };

                let tokens = quote!(
                    pub #field_typed
                );
                Ok(tokens)
            })
            .collect::<syn::Result<Vec<TokenStream>>>()?;

        let tokens = quote!(
            #[derive(Default, Debug)]
            pub struct #data_name {
                #(#fields),*
            }
        );
        Ok(tokens)
    }

    fn generate_builder(&self) -> StreamResult {
        let builder_struct = self.generate_builder_struct();
        let builder_impl = self.generate_builder_impl()?;
        let tokens = quote!(
            #builder_struct
            #builder_impl
        );
        Ok(tokens)
    }

    fn generate_builder_struct(&self) -> TokenStream {
        let data_name = self.info.data_name();
        let builder_name = self.info.builder_name();
        let const_idents = self.generate_builder_const_generic_idents();

        quote!(
            #[derive(Default, Debug)]
            pub struct #builder_name #const_idents {
                data: #data_name
            }
        )
    }

    fn generate_builder_impl(&self) -> StreamResult {
        let builder_new = self.generate_builder_new_impl();
        let builder_setters = self.generate_builder_setters_impl()?;
        let builder_build = self.generate_builder_build_impl();

        let tokens = quote!(
            #builder_new
            #builder_setters
            #builder_build
        );
        Ok(tokens)
    }

    fn generate_builder_new_impl(&self) -> TokenStream {
        let builder_name = self.info.builder_name();
        let consts = self.generate_builder_const_generics_valued(false);

        quote!(
            impl #builder_name #consts {
                pub fn new() -> #builder_name #consts {
                    Self::default()
                }
            }
        )
    }

    fn generate_builder_build_impl(&self) -> TokenStream {
        let target_name = self.info.name();
        let builder_name = self.info.builder_name();
        let group_partials = self.generate_builder_const_generic_group_partial_idents();
        let builder_generics = self.generate_builder_const_generic_idents_final();
        let correctness_verifier = self.generate_builder_group_correctness_verifier();

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

    fn generate_builder_setters_impl(&self) -> StreamResult {
        let field_infos = self.info.field_infos();
        let builder_name = self.info.builder_name();

        let setters = field_infos
            .iter()
            .map(|field| {
                let field_name = field.name();
                let input_name = field.input_name();

                let const_idents_generic = self.generate_builder_const_generic_idents_set_before(field);
                let const_idents_input = self.generate_builder_const_generic_idents_set(field, false);
                let const_idents_output = self.generate_builder_const_generic_idents_set(field, true);

                let (input_typed, input_value) = match field.type_kind()? { // FIXME
                    TypeKind::Mandatory { ty } => (quote!(#input_name: #ty), quote!(Some(#input_name))),
                    TypeKind::MandatoryOption {inner_ty, .. } => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
                    TypeKind::Optional {ty, .. } => (quote!(#input_name: #ty), quote!(#input_name)),
                    TypeKind::GroupOption { inner_ty, .. } => (quote!(#input_name: #inner_ty), quote!(Some(#input_name))),
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
                Ok(tokens)
            })
            .collect::<syn::Result<Vec<TokenStream>>>()?;

        let tokens = quote!(
            #(#setters)*
        );
        Ok(tokens)
    }

    fn generate_builder_const_generic_idents(&self) -> TokenStream {
        let mandatory = (0..self.info.mandatory_count())
            .map(|index| format_ident!("{}_{}", MANDATORY_PREFIX, index));
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count()).map(|index| group.partial_const_ident(index))
        });
        let all = mandatory.chain(groups);
        quote!(<#(const #all: bool),*>)
    }

    fn generate_builder_const_generics_valued(&self, value: bool) -> TokenStream {
        let iter = std::iter::repeat(
            syn::LitBool::new(value, proc_macro2::Span::call_site()).to_token_stream(),
        );
        let mandatory = iter.clone().take(self.info.mandatory_count());
        let groups = self
            .info
            .groups()
            .values()
            .flat_map(|group| iter.clone().take(group.member_count()));
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn generate_builder_const_generic_idents_set(
        &self,
        field_info: &FieldInfo,
        value: bool,
    ) -> TokenStream {
        let mandatory = (0..self.info.mandatory_count()).map(|index| {
            if field_info.mandatory_index() == Some(index) {
                syn::LitBool::new(value, field_info.name().span()).into_token_stream()
            } else {
                format_ident!("{}_{}", MANDATORY_PREFIX, index).into_token_stream()
            }
        });
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count()).map(|index| {
                if field_info.get_group_index(group) == Some(index) {
                    syn::LitBool::new(value, group.name().span()).into_token_stream()
                } else {
                    group.partial_const_ident(index).into_token_stream()
                }
            })
        });
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn generate_builder_const_generic_idents_set_before(
        &self,
        field_info: &FieldInfo,
    ) -> TokenStream {
        let mandatory = (0..self.info.mandatory_count()).filter_map(|index| {
            if field_info.mandatory_index() == Some(index) {
                None
            } else {
                Some(format_ident!("{}_{}", MANDATORY_PREFIX, index).into_token_stream())
            }
        });
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count()).filter_map(|index| {
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

    fn generate_builder_const_generic_idents_final(&self) -> TokenStream {
        let value = syn::LitBool::new(true, proc_macro2::Span::call_site()).to_token_stream();
        let iter = std::iter::repeat(value.clone());
        let mandatory = iter.take(self.info.mandatory_count());
        let groups = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .map(|index| group.partial_const_ident(index).to_token_stream())
        });
        let all = mandatory.chain(groups);
        quote!(<#(#all),*>)
    }

    fn generate_builder_const_generic_group_partial_idents(&self) -> TokenStream {
        let all = self.info.groups().values().flat_map(|group| {
            (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream())
        });
        quote!(<#(const #all: bool),*>)
    }

    fn generate_builder_group_correctness_verifier(&self) -> TokenStream {
        let all = self.info.groups().values().flat_map(|group| {
            let partials = (0..group.member_count())
                .map(|index| group.partial_const_ident(index).into_token_stream());
            let function_call = group.function_ident();
            let count = group.expected_count();
            let name = group.name();
            let err_text = format!("Group {name} not verified");
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
}
