use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[groups(baz = at_least(2))]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
        #[builder(group = baz)]
        baz: Option<String>,
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).build();
}