use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[group(baz = at_least(2))]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
    }
    let foobuilder = Foo::builder().bar("Hello world!".to_string());
}