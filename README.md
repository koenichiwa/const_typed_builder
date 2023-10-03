# `Builder` Derive Macro Documentation

The `Builder` derive macro is used to generate builder methods for structs in Rust, its biggest feature in this crate is that it provides compile-time validation on the struct. The user can employ several configurations that define the validity of of a complex struct, and it is checked before the struct is ever created. 

The library also checks cases where the struct can never be valid or is always valid, but this is still in a work in progress. Errors are always emitted, but warnings are emitted on the nightly channel only. This is due to a limitation in [proc_macro_error](https://docs.rs/proc-macro-error/latest/proc_macro_error/).

## Prerequisites

To use the `Builder` derive macro, you should have the `const_typed_builder` crate added to your project's dependencies in your `Cargo.toml` file:

```toml
[dependencies]
const_typed_builder = "0.3"
```

Also, make sure you have the following use statement in your code:

```rust
use const_typed_builder::Builder;
```
## Overview
The crate can be used to check validity of structs at compile time. The user can't call build until the struct is valid. This is done by checking if all mandatory fields are instantiated and all user defined "groups" are valid.

### Examples
Basic usage:
```rust
use const_typed_builder::Builder;

#[derive(Debug, Builder)]
pub struct Foo {
    bar: String,
}

let foo = Foo::builder()
    .bar("Hello world!".to_string()) // <- The program would not compile without this call
    .build();                        // <- Because this function is only implemented for the
                                     // .. version of FooBuilder where `bar` is initialized
```

Subset of launchd:
```rust
use const_typed_builder::Builder;
use std::path::PathBuf;

#[derive(Debug, Builder)]
pub struct ResourceLimits {
    core: Option<u64>,
    // ...
}
#[derive(Debug, Builder)]
#[group(program = at_least(1))]
pub struct Launchd {
    #[builder(mandatory)]
    label: Option<String>,
    disabled: Option<bool>,
    user_name: Option<String>,
    group_name: Option<String>,
    // ...
    #[builder(group = program)]
    program: Option<PathBuf>,
    bundle_program: Option<String>,
    #[builder(group = program)]
    program_arguments: Option<Vec<String>>,
    // ...
    #[builder(skip)]
    on_demand: Option<bool>, // NB: deprecated (see KeepAlive), but still needed for reading old plists.
    #[builder(skip)]
    service_ipc: Option<bool>, // NB: "Please remove this key from your launchd.plist."
    // ...
    #[builder(propagate)]
    soft_resource_limits: Option<ResourceLimits>,
    #[builder(propagate)]
    hard_resource_limits: Option<ResourceLimits>,
    // ...
}

let launchd = Launchd::builder()
    .label("my_label".to_string())    // <- 1: Mandatory
    .program("./my_program".into())   // <- 2: Launchd expects that least one of these fields is set..
    .program_arguments(               // <- 2: .. We can remove either one, but never both
        vec!["my_arg".to_string()]
    ) 
//  .on_demand(false)                    <- 3: This function doesn't exist
    .soft_resource_limits(|builder|
        Some(builder.core(Some(1)).build()) // <- 4: Propagating to `ResourceLimits::builder`
    ) 
    .build();
```

### Attributes
**Struct**
- `#[builder(assume_mandatory)]`: Indicates that all fields in the struct should be assumed as mandatory.
  If provided without an equals sign (e.g., `#[builder(assume_mandatory)]`), it sets the `mandatory` flag for fields to true.
  If provided with an equals sign (e.g., `#[builder(assume_mandatory = true)]`), it sets the `mandatory` flag for fields based on the value.
- `#[group(group_name = (exact(N)|at_least(N)|at_most(N)|single)]`:
  Associates fields of the struct with a group named "group_name" and specifies the group's behavior.
  The `group_name` should be a string identifier. The group can have one of the following behaviors:
    - `exact(N)`: Exactly N fields in the group must be set during the builder construction.
    - `at_least(N)`: At least N fields in the group must be set during the builder construction.
    - `at_most(N)`: At most N fields in the group can be set during the builder construction.
    - `single`: Only one field in the group can be set during the builder construction. This is a shorthand for `exact(1)`.
  e.g `#[group(foo = at_least(2))]` creates a group where at least 2 of the fields need to be initialized.
- `#[builder(solver = (brute_force|compiler))]`: **Use sparingly, see note at bottom of this file!** 
   Specifies the solver type to be used for building the struct. The `solve_type` should be one of the predefined solver types, such as `brute_force` or `compiler`. If provided with an equals sign (e.g., `#[builder(solver = brute_force)]`),
   it sets the "solver type" accordingly. This attribute is still tested, and `brute_force` is the default, and only if there are problems in compilation time then you can try `compiler`. `compiler` gives less guarantees though.
 
**Field**
- `#[builder(group = group_name)]`: The heart of this library. This associates the field with a group named `group_name`.
  Fields in the same group are treated as a unit, and at least one of them must be set during builder construction. This attribute allows specifying the group name both as an identifier (e.g., `group = my_group`)
  and as a string (e.g., `group = "my_group"`).
- `#[builder(mandatory)]`: Marks the field as mandatory, meaning it must be set during the builder
  construction. If provided without an equals sign (e.g., `#[builder(mandatory)]`), it sets the field as mandatory.
  If provided with an equals sign (e.g., `#[builder(mandatory = true)]`), it sets the mandatory flag based on the value.
- `#[builder(optional)]`: Marks the field as optional, this is the exact opposite of `#[builder(mandatory)]`.
  If provided without an equals sign (e.g., `#[builder(optional)]`), it sets the field as optional.
  If provided with an equals sign (e.g., `#[builder(optional = true)]`), it sets the optional flag based on the value.
- `#[builder(skip)]`: Marks the field as skipped, meaning that the builder will not include it. This can be used for
  fields that are deprecated, but must still be deserialized. This way you can ensure that new structs will never be created with this field initialized, but that old structs can still be used. The field type has to be `Option<T>` for this to work.
- `#[builder(propagate)]`: Indicates that the field should propagate its value when the builder is constructed. 
  If this attribute is present, the field's value will be copied or moved to the constructed object when the builder is used to build the object.

Fields can either be a part of a group, mandatory, optional OR skipped. These attribute properties are mutually exclusive. `propagate` can be used on any field where the type also derives `Builder`.

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
