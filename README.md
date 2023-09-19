# `Builder` Derive Macro Documentation

 The `Builder` derive macro is used to generate builder methods for structs in Rust. These builder methods allow you to construct instances of the struct by chaining method calls, providing a convenient and readable way to create complex objects with various configurations and compile-time validity checking. This documentation will provide an overview of how to use the `Builder` derive macro.

 ## Prerequisites

To use the `Builder` derive macro, you should have the `const_typed_builder` crate added to your project's dependencies in your `Cargo.toml` file:

```toml
[dependencies]
const_typed_builder = "0.2"
```

Also, make sure you have the following import statements in your code:

```rust
use const_typed_builder::Builder;
```
## Effects 
This derive:
```rust
use const_typed_builder::Builder;
#[derive(Debug, Builder)]
pub struct Foo {
    bar: String,
}
```
Expands to this code:
```rust
use const_typed_builder::Builder;
#[derive(Debug)]
pub struct Foo {
    bar: String,
}
impl Builder for Foo {
    type BuilderImpl = FooBuilder<false>;
    fn builder() -> Self::BuilderImpl {
        Self::BuilderImpl::new()
    }
}
#[derive(Debug)]
pub struct FooBuilder<const M_0: bool> {
    data: FooData,
}
impl FooBuilder<false> {
    pub fn new() -> FooBuilder<false> {
        Self::default()
    }
}
impl Default for FooBuilder<false> {
    fn default() -> Self {
        FooBuilder {
            data: FooData::default(),
        }
    }
}
impl FooBuilder<false> {
    pub fn bar(self, bar: String) -> FooBuilder<true> {
        let mut data = self.data;
        data.bar = Some(bar);
        FooBuilder { data }
    }
}
impl FooBuilder<true> {
    pub fn build(self) -> Foo {
        self.data.into()
    }
}
#[derive(Debug)]
pub struct FooData {
    pub bar: Option<String>,
}
impl From<FooData> for Foo {
    fn from(data: FooData) -> Foo {
        Foo {
            bar: data.bar.unwrap(),
        }
    }
}
impl Default for FooData {
    fn default() -> Self {
        FooData { bar: None }
    }
}
```
## Inspirations
Builder macros have been done before, but not exactly what I needed for my use case. Also look into [derive_builder](https://crates.io/crates/derive_builder) and [typed-builder](https://crates.io/crates/typed-builder). Those projects are currently way more mature, but anyone willing to test this crate is currently a godsend.

> [!NOTE]
> The current default implementation for checking the validity of grouped fields is `brute_force`. The problem is directly related to [SAT](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem), and the `brute_force` implementation currently has a $`O(2^g)`$ where $`g`$ is the amount of grouped variables. This is not a problem with a couple of fields, but it might impact compile time significantly with more fields. Future editions might improve on this.
>
> Although I haven't been able to recreate the issue yet, it seems that const values [aren't guaranteed to be evaluated at compile time](https://doc.rust-lang.org/reference/const_eval.html). This creates the issue that the group verification is not guaranteed to fail in its current implementation when the solver type is set to `compiler`. Users can opt in to the `compiler` solver, which might solve it quicker than `brute_force`, although this is not guaranteed.
>
> Anyone who would like to help, and add a SAT solver as a dependency (behind a feature flag) is welcome to do so.
