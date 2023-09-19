use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        bar: String,
        baz: String,
        qux: String,
        quz: String,
    }

    let foo = Foo::builder()
        .bar("Hello".to_string())
        .baz("world!".to_string())
        .qux("Hello".to_string())
        .build();
}