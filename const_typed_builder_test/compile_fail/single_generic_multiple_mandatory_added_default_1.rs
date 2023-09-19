use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo<A>
    where
        A: Default,
    {
        bar: A,
        baz: A,
    }
    let foo = Foo::builder().build();
}