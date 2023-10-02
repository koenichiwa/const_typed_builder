use const_typed_builder::Builder;
fn main() {
    #[derive(Debug, PartialEq, Builder)]
    pub struct Foo {
        bar: String,
        #[builder(skip)]
        baz: Option<String>,
    }
    let foo = Foo::builder().bar("Hello world!".to_string()).baz("Skipped".to_string()).build();
}