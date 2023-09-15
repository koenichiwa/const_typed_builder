use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        bar: String,
        baz: String,
        qux: Option<String>,
        quz: Option<String>,
    }
    
    let foo = Foo::builder()
        .bar("Hello".to_string())
        .qux(Some("Hello".to_string()))
        .quz(Some("world!".to_string()))
        .build();
}