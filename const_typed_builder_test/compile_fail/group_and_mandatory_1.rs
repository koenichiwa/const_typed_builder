use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[group(quz = single)]
    pub struct Foo {
        #[builder(group = quz)]
        bar: Option<String>,
        #[builder(group = quz)]
        baz: Option<String>,
        qux: String,
    }

    let _ = Foo::builder().baz("Hello".to_string()).build();
}