mod builder_info;
mod data_info;
mod field_info;
mod generator;
mod struct_info;
mod util;

use generator::Generator;
use proc_macro2::TokenStream;
use syn::DeriveInput;

type StreamResult = Result<TokenStream, syn::Error>;
type VecStreamResult = Result<Vec<TokenStream>, syn::Error>;

const MANDATORY_NAME: &str = "M";

#[proc_macro_derive(Builder, attributes(mandatory))]
pub fn derive_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    match impl_my_derive(&ast) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn impl_my_derive(ast: &syn::DeriveInput) -> StreamResult {
    let generator = Generator::new(ast)?;
    generator.generate()
}
