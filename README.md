# `Builder` Derive Macro Documentation

 The `Builder` derive macro is used to generate builder methods for structs in Rust, its biggest feature in this crate is that it provides compile-time validation on the struct. The user can employ several configurations that define the validity of of a complex struct, and it is checked before the struct is ever created.

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
## Overview
The crate can be used to check validity of structs at compile time. The user can't call build until the struct is valid. This is done by checking if all mandatory fields are instantiated and all user defined "groups" are valid.

### Simple example
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

### Attributes
**Struct**
- `#[builder(assume_mandatory)]`: Indicates that all fields in the struct should be assumed as mandatory.
  If provided without an equals sign (e.g., `#[builder(assume_mandatory)]`), it sets the `mandatory` flag for fields to true.
  If provided with an equals sign (e.g., `#[builder(assume_mandatory = true)]`), it sets the `mandatory` flag for fields based on the value.
- `#[builder(solver = `solve_type`)]`: Specifies the solver type to be used for building the struct. The `solve_type` should be one of
   the predefined solver types, such as `brute_force` or `compiler`. If provided with an equals sign (e.g., `#[builder(solver = brute_force)]`),
   it sets the `solver_type` accordingly.
- `#[group(group_name = (exact(N)|at_least(N)|at_most(N)|single)]`:
  Associates fields of the struct with a group named "group_name" and specifies the group's behavior.
  The `group_name` should be a string identifier. The group can have one of the following behaviors:
    - `exact(N)`: Exactly N fields in the group must be set during the builder construction.
    - `at_least(N)`: At least N fields in the group must be set during the builder construction.
    - `at_most(N)`: At most N fields in the group can be set during the builder construction.
    - `single`: Only one field in the group can be set during the builder construction.
 
**Field**
- `#[builder(mandatory)]`: Marks the field as mandatory, meaning it must be set during the builder
  construction. If provided without an equals sign (e.g., `#[builder(mandatory)]`), it sets the field as mandatory.
  If provided with an equals sign (e.g., `#[builder(mandatory = true)]`), it sets the mandatory flag based on the value.
- `#[builder(optional)]`: Marks the field as optional, this is the exact opposite of mandatory.
  If provided without an equals sign (e.g., `#[builder(optional)]`), it sets the field as optional.
  If provided with an equals sign (e.g., `#[builder(optional = true)]`), it sets the optional flag based on the value.
- `#[builder(group = group_name)]`: Associates the field with a group named `group_name`. Fields in the same group
  are treated as a unit, and at least one of them must be set during builder construction. If the field is marked as mandatory,
  it cannot be part of a group. This attribute allows specifying the group name both as an identifier (e.g., `group = my_group`)
  and as a string (e.g., `group = "my_group"`).
- `#[builder(propagate)]`: Indicates that the field should propagate its value when the builder is constructed. If this attribute
  is present, the field's value will be copied or moved to the constructed object when the builder is used to build the object.

> [!NOTE]
> Checking the validity of each field is a problem directly related to [SAT](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem), which is an NP-complete problem. This has effect especially the grouped fields. The current default implementation for checking the validity of grouped fields is `brute_force`, and this implementation currently has a complexity of $`O(2^g)`$ where $`g`$ is the amount of grouped variables. This is not a problem with a couple of fields, but it might impact compile time significantly with more fields. This can be still optimized significantly. Future editions might improve on this complexity.
>
> Another implementation is `compiler`. I haven't tested its speed increase yet, but it might have an issue. Although I haven't been able to recreate the issue yet, it seems that const values [aren't guaranteed to be evaluated at compile time](https://doc.rust-lang.org/reference/const_eval.html). This creates the issue that the group verification is not guaranteed to fail during compile-time. 
> 
> Users can opt in to the `compiler` solver, by adding `#[builder(solver = compiler)]` above the struct. I'm not making any guarantees on its performance.
>
> Anyone who would like to help, and add a SAT solver as a dependency (behind a feature flag) is welcome to do so!

## Inspirations
Builder macros have been done before, but not exactly what I needed for my use case. Also look into [derive_builder](https://crates.io/crates/derive_builder) and [typed-builder](https://crates.io/crates/typed-builder). Those projects are currently way more mature, but anyone willing to test this crate is currently a godsend.
