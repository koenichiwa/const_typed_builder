mod generator;
mod info;
mod parser;
mod symbol;
mod util;

use generator::Generator;
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::DeriveInput;

/// The `derive_builder` macro is used to automatically generate builder
/// code for a struct. It takes a struct as input and generates a builder
/// pattern implementation for that struct.
///
/// # Example
///
/// ```ignore
/// #[derive(Builder)]
/// struct MyStruct {
///     field1: i32,
///     field2: String,
/// }
/// ```
///
/// This will generate a builder pattern for `MyStruct`, allowing you to
/// construct instances of `MyStruct` with a fluent API.
#[proc_macro_derive(Builder, attributes(builder, group))]
#[proc_macro_error]
pub fn derive_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let stream = impl_my_derive(&ast);
    quote!(#stream).into()
}

/// This function implements the custom derive for the `Builder` trait.
///
/// It takes a `syn::DeriveInput` as input, which represents the struct
/// for which the builder pattern is being generated. It then extracts
/// information about the struct and uses a `Generator` to generate the
/// builder pattern code.
///
/// # Arguments
///
/// - `ast`: A `syn::DeriveInput` representing the input struct.
///
/// # Returns
///
/// An optional `TokenStream` representing the generated token stream.
fn impl_my_derive(ast: &syn::DeriveInput) -> Option<TokenStream> {
    let container_info = parser::ContainerParser::new().parse(ast)?;
    let generator = Generator::new(&container_info);
    Some(generator.generate())
}
