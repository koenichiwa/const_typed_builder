use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[groups(quz = single)]
    pub struct Foo {
        #[builder(group = quz)]
        bar: Option<String>,
        #[builder(group = quz)]
        #[builder(mandatory)]
        baz: Option<String>,
        qux: String,
    }

    let _ = Foo::builder().bar("Hello".to_string()).baz("Hello".to_string()).qux("world!".to_string()).build();
}