use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[groups(baz = single)]
    pub struct Foo {
        #[builder(group = baz)]
        bar: Option<String>,
        #[builder(group = baz)]
        qux: Option<String>,
    }
    _ = Foo::builder().bar("Hello world!".to_string()).qux("Hello world!".to_string()).build();
}