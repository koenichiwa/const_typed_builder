use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[builder(assume_mandatory)]
    pub struct Foo {
        bar: Option<String>,
        baz: Option<String>,
        #[builder(optional)]
        quz: Option<String>,
    }
    let foo = Foo::builder()
        .bar("Hello world!".to_string())
        .baz("Hello world!".to_string())
        .quz("Hello world!".to_string())
        .build();
}