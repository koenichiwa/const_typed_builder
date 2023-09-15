extern crate const_typed_builder;
use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[builder(assume_mandatory)]
    pub struct Foo {
        bar: Option<String>,
    }
    let foo = Foo::builder().bar(Some("Hello world!".to_string())).build(); //~ ERROR E0599
}