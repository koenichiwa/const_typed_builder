<!-- # const_typed_builder -->
# `Builder` Derive Macro Documentation

 The `Builder` derive macro is used to generate builder methods for structs in Rust. These builder methods allow you to construct instances of the struct by chaining method calls, providing a convenient and readable way to create complex objects with various configurations and compile-time validity checking. This documentation will provide an overview of how to use the `Builder` derive macro.

 ## Prerequisites

To use the `Builder` derive macro, you should have the `const_typed_builder` crate added to your project's dependencies in your `Cargo.toml` file:

```toml
[dependencies]
const_typed_builder = "0.1"
```

Also, make sure you have the following import statements in your code:

```rust
use const_typed_builder::Builder;
```
## Inspirations
Builder macros have been done before, but not exactly what I needed for my use case. Also look into [derive_builder](https://crates.io/crates/derive_builder) and [typed-builder](https://crates.io/crates/typed-builder). Those projects are currently way more mature, but anyone willing to test this crate is currently a godsend.