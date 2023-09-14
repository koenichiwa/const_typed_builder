use const_typed_builder::Builder;

fn main() {
    #[derive(Debug, Default, PartialEq, Eq, Builder)]
    pub struct Foo {
        #[builder(mandatory)]
        bar: Option<String>,
    }
    
    let foo = Foo::builder().bar(Some("Hello world!".to_string())).build();
}