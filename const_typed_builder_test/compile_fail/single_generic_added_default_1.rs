use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo<A>
    where
        A: Default,
    {
        bar: A,
    }

    let foo: Foo<Option<String>> = Foo::builder().build();
}