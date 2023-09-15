use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        bar: Option<String>,
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).build();
}