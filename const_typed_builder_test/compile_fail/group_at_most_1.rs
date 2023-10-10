use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    #[groups(quz = at_most(2))]
    pub struct Foo {
        #[builder(group = quz)]
        bar: Option<String>,
        #[builder(group = quz)]
        baz: Option<String>,
        #[builder(group = quz)]
        qux: Option<String>,
    }

     let foo = Foo::builder().qux("!".to_string()).baz("world".to_string()).bar("Hello".to_string()).build();
}