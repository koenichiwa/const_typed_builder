extern crate const_typed_builder;
use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        bar: String,
    }
    
    let foo = Foo::builder().build(); //~ ERROR E0599
}