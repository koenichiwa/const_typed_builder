// #![allow(unused, unused_variables, dead_code)]
// mod field_info;
mod generator;
// mod group_info;
// mod struct_info;
mod info;
mod symbol;
mod util;

use generator::Generator;
use info::StructInfo;
use proc_macro2::TokenStream;
use syn::DeriveInput;

type StreamResult = syn::Result<TokenStream>;
type VecStreamResult = Result<Vec<TokenStream>, syn::Error>;

const MANDATORY_PREFIX: &str = "M";

#[proc_macro_derive(Builder, attributes(builder, group))]
pub fn derive_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    impl_my_derive(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn impl_my_derive(ast: &syn::DeriveInput) -> StreamResult {
    let struct_info = StructInfo::new(ast)?;
    let generator = Generator::new(&struct_info);
    generator.generate()
}
