use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        #[builder(propagate)]
        bar: Option<Bar>,
    }

    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Bar {
        baz: String,
    }

    let foo = Foo::builder().bar(|builder| builder.build() ).build();
}