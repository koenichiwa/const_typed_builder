// This code is not compiling but trybuild thinks it does
use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[group(baz = at_least(2))]
    #[build(solver = brute_force)]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).build();
}