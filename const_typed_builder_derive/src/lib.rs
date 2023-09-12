mod context;
mod field_info;
mod generator;
mod group;
mod struct_info;
mod symbol;
mod util;

use context::Context;
use generator::Generator;
use proc_macro2::TokenStream;
use struct_info::StructInfo;
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
    let mut context = Context::new();
    let struct_info = StructInfo::new(&mut context, ast).ok_or_else(|| {
        context
            .get_error()
            .unwrap_or_else(|| syn::Error::new_spanned(ast, "Unknown error during parsing"))
    })?;
    let generator = Generator::new(struct_info);
    generator.generate(&mut context).ok_or_else(|| {
        context
            .get_error()
            .unwrap_or_else(|| syn::Error::new_spanned(ast, "Unknown error during generating"))
    })
}
