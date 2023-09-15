extern crate const_typed_builder;
use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)] //~ ERROR E0080
    #[group(baz = at_least(2))]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).build(); // TODO: Get error to throw here
}