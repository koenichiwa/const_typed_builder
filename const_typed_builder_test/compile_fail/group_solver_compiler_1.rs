// This code is not compiling but trybuild thinks it does
use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[group(baz = at_least(2))]
    #[builder(solver = compiler)]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
        #[builder(group = baz)]
        baz: Option<String>
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).build();
}