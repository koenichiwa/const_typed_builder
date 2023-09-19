use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo<A, B> {
        bar: A,
        baz: B,
    }

     let foo: Foo<String, usize> = Foo::builder().bar("Hello world!".to_string()).baz("Hello world!".to_string()).build();
}