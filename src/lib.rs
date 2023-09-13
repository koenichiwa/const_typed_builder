#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
/// The `Builder` derive macro provides a convenient and ergonomic way to create builder patterns for struct types in Rust. 
/// It allows you to generate builder methods for constructing instances of a struct with various features and constraints, all checked at compile time. 
/// Below are the highlighted features and explanations for each, along with examples:
/// # Features
/// ## 1. Compile-Time Validity Checking
///
/// The `Builder` derive macro ensures that the constructed struct is valid at compile time. 
/// This means that it checks that all required fields are provided and that group constraints are satisfied. 
/// If any constraints are violated, the code will not compile.
///
/// ### Examples:
/// Basic example
/// ```rust
/// use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     bar: String,
///     baz: String,
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string()) // Mandatory field
///     .baz("Hello".to_string()) // Mandatory field
///     .build();
/// ```
///  
/// This will not compile without providing 'baz'
/// ```compile_fail
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     bar: String,
///     baz: String,
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string()) // Mandatory field
///     .build();
/// ```
///
/// ## 2. Mandatory and Optional Fields
///
/// By default, all fields in the generated builder are considered mandatory, meaning they must be provided during construction.
/// However, fields with the `Option` type are considered optional and can be left unset, and thus defaulted to `None`.
///
/// ### Example:
/// Valid construction with optional field left unset
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     bar: String,            // Mandatory
///     baz: Option<String>,    // Optional
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string())
///     .build();
/// ```
///  
/// You can also specify that an `Option` field is in fact mandatory by using the attribute `mandatory`
///  
/// ### Example:
/// Invalid construction with optional field left unset:
/// ```compile_fail
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     bar: String,            // Mandatory
///     #[builder(mandatory)]
///     baz: Option<String>,    // Mandatory (but of type `Option`)
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string())
///     .build();
/// ```
///
/// ## 3. Grouping Fields
///
/// Fields can be grouped together, and constraints can be applied to these groups. Groups allow you to ensure that a certain combination of fields is provided together. 
/// There are four types of groups: `single`, `at_least`, `at_most`, and `exact`.
/// 
/// **All** fields that are grouped need to be an `Option` type.
///
/// - `single`: Ensures that only one field in the group can be provided. (It's basically a shorthand for `exact(1)`)
/// - `at_least(n)`: Requires at least `n` fields in the group to be provided.
/// - `at_most(n)`: Allows at most `n` fields in the group to be provided.
/// - `exact(n)`: Requires exactly `n` fields in the group to be provided.
///
/// ### Examples:
///
/// Valid construction only one field in 'my_group' is provided
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// #[group(my_group = single)]
/// pub struct Foo {
///     #[builder(group = my_group)]
///     bar: Option<String>,
///     #[builder(group = my_group)]
///     baz: Option<String>,
/// }
///
/// let foo = Foo::builder()
///     .baz("World".to_string())
///     .build();
/// ```
/// Invalid construction because both fields in 'my_group' are provided
/// ```compile_fail
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// #[group(my_group = single)]
/// pub struct Foo {
///     #[builder(group = my_group)]
///     bar: Option<String>,
///     #[builder(group = my_group)]
///     baz: Option<String>,
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string())
///     .baz("World".to_string())
///     .build();
/// ```
/// Valid construction because at least 2 fields in 'my_group' are provided:
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// #[group(my_group = at_least(2))]
/// pub struct Foo {
///     #[builder(group = my_group)]
///     bar: Option<String>,
///     #[builder(group = my_group)]
///     baz: Option<String>,
///     #[builder(group = my_group)]
///     qux: Option<String>,
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string())
///     .baz("World".to_string())
///     .build();
/// ```
/// You can also add multiple groups to each field
///
/// Valid construction because at least 2 fields in 'least' are provided, and 'fred' had to be provided to validate the group 'most':
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// #[group(least = at_least(2))]
/// #[group(most = at_most(3))]
/// pub struct Foo {
///     #[builder(group = least)]
///     #[builder(group = most)]
///     bar: Option<String>,
///     #[builder(group = least)]
///     #[builder(group = most)]
///     baz: Option<String>,
///     #[builder(group = least)]
///     #[builder(group = most)]
///     qux: Option<String>,
///     #[builder(group = least)]
///     #[builder(group = most)]
///     quz: Option<String>,
///     #[builder(group = most)]
///     fred: Option<String>,
/// }
///
/// let foo = Foo::builder()
///     .bar("Hello".to_string())
///     .baz("World".to_string())
///     .fred("!".to_string())
///     .build();
/// ```
///
/// ## 4. Propagating Builder for Complex Structs
///
/// If a field in the struct is of a complex type that also derives the `Builder` trait, you can propagate the construction of that field to its builder using the `propagate` attribute.
///
/// *Disclaimer* I'm still working a lot on this feature and I may change how it works. (I would love for the user to return the builder at the end of the function instead of calling `.builder()` by hand)
///
/// ### Example:
/// Valid construction with complex struct 'Bar' created within 'Foo'
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     #[builder(propagate)]
///     bar: Bar,
/// }
///
/// #[derive(Debug, Builder)]
/// pub struct Bar {
///     baz: String,
/// }
///
/// let foo = Foo::builder()
///     .bar(|builder| builder.baz("Hello world!".to_string()).build())
///     .build();
/// ```
///
/// ## 6. Generic Structs
///
/// The `Builder` derive macro supports generic structs, allowing you to generate builders for generic types with constraints, such as default values for generic parameters.
///
/// *Disclaimer* I haven't finished testing this feature yet
///
/// ### Example:
/// Valid construction for a generic struct 'Foo' with a default generic parameter
/// ```rust
/// # use const_typed_builder::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo<A>
/// where
///     A: Default,
/// {
///     bar: A,
/// }
///
/// let foo = Foo::<String>::builder()
///     .bar("Hello world!".to_string())
///     .build();
/// ```
///
/// These are the key features and explanations of the `Builder` derive macro in Rust, along with examples illustrating each feature. This macro simplifies the process of creating builders for your structs while ensuring compile-time safety and correctness.
pub use const_typed_builder_derive::Builder;
/// The `Builder` trait facilitates the creation of builder patterns for Rust struct types. It provides a common interface for generating builders that enable the construction of instances of a struct with various configurations and compile-time validity checking.
pub trait Builder {
    type BuilderImpl;
    fn builder() -> Self::BuilderImpl;
}

#[cfg(test)]
mod test {
    use super::Builder;

    #[test]
    fn single_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            bar: String,
        }

        let expected = Foo {
            bar: "Hello world!".to_string(),
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().build();
    }

    #[test]
    fn option_not_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            bar: Option<String>,
        }
        let foo = Foo::builder().build();
        let expected = Foo { bar: None };
        assert_eq!(expected, foo);

        let foo = Foo::builder().bar(None).build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar(Some("Hello world!".to_string())).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn multiple_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            bar: String,
            baz: String,
            qux: String,
            quz: String,
        }

        let expected = Foo {
            bar: "Hello".to_string(),
            baz: "world!".to_string(),
            qux: "Hello".to_string(),
            quz: "world!".to_string(),
        };
        let foo = Foo::builder()
            .bar("Hello".to_string())
            .baz("world!".to_string())
            .qux("Hello".to_string())
            .quz("world!".to_string())
            .build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn mixed_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            bar: String,
            baz: String,
            qux: Option<String>,
            quz: Option<String>,
        }

        let expected = Foo {
            bar: "Hello".to_string(),
            baz: "world!".to_string(),
            qux: Some("Hello".to_string()),
            quz: Some("world!".to_string()),
        };
        let foo = Foo::builder()
            .bar("Hello".to_string())
            .baz("world!".to_string())
            .qux(Some("Hello".to_string()))
            .quz(Some("world!".to_string()))
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: "Hello".to_string(),
            baz: "world!".to_string(),
            qux: Some("Hello".to_string()),
            quz: None,
        };
        let foo = Foo::builder()
            .bar("Hello".to_string())
            .baz("world!".to_string())
            .qux(Some("Hello".to_string()))
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: "Hello".to_string(),
            baz: "world!".to_string(),
            qux: None,
            quz: None,
        };
        let foo = Foo::builder()
            .bar("Hello".to_string())
            .baz("world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let foo = Foo::builder()
            .bar("Hello".to_string())
            .baz("world!".to_string())
            .qux(None)
            .quz(None)
            .build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn option_explicit() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            bar: std::option::Option<String>,
        }
        let foo = Foo::builder().build();
        let expected = Foo { bar: None };
        assert_eq!(expected, foo);

        let foo = Foo::builder().bar(None).build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar(Some("Hello world!".to_string())).build();
        assert_eq!(expected, foo);

        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Bar {
            baz: core::option::Option<String>,
        }
        let bar = Bar::builder().build();
        let expected = Bar { baz: None };
        assert_eq!(expected, bar);

        let bar = Bar::builder().baz(None).build();
        assert_eq!(expected, bar);

        let expected = Bar {
            baz: Some("Hello world!".to_string()),
        };
        let bar = Bar::builder().baz(Some("Hello world!".to_string())).build();
        assert_eq!(expected, bar);
    }

    #[test]
    fn optional_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            #[builder(mandatory)]
            bar: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn optional_mandatory_set() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            #[builder(mandatory = true)]
            bar: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn group() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(baz = single)]
        pub struct Foo {
            #[builder(group = baz)]
            bar: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn group_multiple_member() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(baz = single)]
        pub struct Foo {
            #[builder(group = baz)]
            bar: Option<String>,
            #[builder(group = baz)]
            qux: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
            qux: None,
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            qux: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().qux("Hello world!".to_string()).build();
        assert_eq!(expected, foo);

        // // EXPECT: DOESNT RUN
        // let expected = Foo {
        //     bar: Some("Hello world!".to_string()),
        //     qux: Some("Hello world!".to_string()),
        // };
        // let foo = Foo::builder().bar("Hello world!".to_string()).qux("Hello world!".to_string()).build();
        // assert_eq!(expected, foo);
    }

    #[test]
    fn group_and_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(quz = single)]
        pub struct Foo {
            #[builder(group = quz)]
            bar: Option<String>,
            #[builder(group = quz)]
            baz: Option<String>,
            qux: String,
        }

        let expected = Foo {
            bar: Some("Hello".to_string()),
            baz: None,
            qux: "world!".to_string(),
        };

        let foo = Foo::builder()
            .bar("Hello".to_string())
            .qux("world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            baz: Some("Hello".to_string()),
            qux: "world!".to_string(),
        };

        let foo = Foo::builder()
            .qux("world!".to_string())
            .baz("Hello".to_string())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().baz("Hello".to_string()).build();
        // let foo = Foo::builder().bar("Hello".to_string()).baz("Hello".to_string()).qux("world!".to_string()).build();
    }

    #[test]
    fn group_and_option_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(quz = single)]
        pub struct Foo {
            #[builder(group = quz)]
            bar: Option<String>,
            #[builder(group = quz)]
            baz: Option<String>,
            #[builder(mandatory)]
            qux: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello".to_string()),
            baz: None,
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .bar("Hello".to_string())
            .qux("world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            baz: Some("Hello".to_string()),
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .qux("world!".to_string())
            .baz("Hello".to_string())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().baz("Hello".to_string()).build();
        // let foo = Foo::builder().bar("Hello".to_string()).baz("Hello".to_string()).qux("world!".to_string()).build();

        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(quz = single)]
        pub struct Nope {
            #[builder(group = quz)]
            bar: Option<String>,
            #[builder(group = quz)]
            #[builder(mandatory)]
            baz: Option<String>,
            #[builder(mandatory)]
            qux: Option<String>,
        }
    }

    #[test]
    fn group_at_least() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(quz = at_least(2))]
        pub struct Foo {
            #[builder(group = quz)]
            bar: Option<String>,
            #[builder(group = quz)]
            baz: Option<String>,
            #[builder(group = quz)]
            qux: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello".to_string()),
            baz: None,
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .bar("Hello".to_string())
            .qux("world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            baz: Some("Hello".to_string()),
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .qux("world!".to_string())
            .baz("Hello".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: Some("Hello".to_string()),
            baz: Some("world".to_string()),
            qux: Some("!".to_string()),
        };

        let foo = Foo::builder()
            .qux("!".to_string())
            .baz("world".to_string())
            .bar("Hello".to_string())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().baz("Hello".to_string()).build();
        // let foo = Foo::builder().bar("Hello".to_string()).bar("Hello".to_string()).qux("world!".to_string()).build();
    }

    #[test]
    fn group_at_most() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(quz = at_most(2))]
        pub struct Foo {
            #[builder(group = quz)]
            bar: Option<String>,
            #[builder(group = quz)]
            baz: Option<String>,
            #[builder(group = quz)]
            qux: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello".to_string()),
            baz: None,
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .bar("Hello".to_string())
            .qux("world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            baz: Some("Hello".to_string()),
            qux: Some("world!".to_string()),
        };

        let foo = Foo::builder()
            .qux("world!".to_string())
            .baz("Hello".to_string())
            .build();
        assert_eq!(expected, foo);

        let expected = Foo {
            bar: None,
            baz: Some("Hello world!".to_string()),
            qux: None,
        };
        let foo = Foo::builder().baz("Hello world!".to_string()).build();
        assert_eq!(expected, foo);

        // let expected = Foo {
        //     bar: Some("Hello".to_string()),
        //     baz: Some("world".to_string()),
        //     qux: Some("!".to_string()),
        // };

        // let foo = Foo::builder().qux("!".to_string()).baz("world".to_string()).bar("Hello".to_string()).build();
        // assert_eq!(expected, foo);

        // let foo = Foo::builder().bar("Hello".to_string()).bar("Hello".to_string()).qux("world!".to_string()).build();
    }

    #[test]
    fn single_generic_added_default() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo<A>
        where
            A: Default,
        {
            bar: A,
        }

        let expected = Foo::<String> {
            bar: "Hello world!".to_string(),
        };
        let foo = Foo::<String>::builder()
            .bar("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let foo: Foo<String> = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn single_generic_multiple_mandatory_added_default() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo<A>
        where
            A: Default,
        {
            bar: A,
            baz: A,
        }

        let expected = Foo::<String> {
            bar: "Hello world!".to_string(),
            baz: "Hello world!".to_string(),
        };
        let foo = Foo::<String>::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let foo: Foo<String> = Foo::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().build();
    }

    #[test]
    fn single_generic_multiple_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo<A> {
            bar: A,
            baz: A,
        }

        let expected = Foo::<String> {
            bar: "Hello world!".to_string(),
            baz: "Hello world!".to_string(),
        };
        let foo = Foo::<String>::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);

        let foo: Foo<String> = Foo::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().build();
    }

    #[test]
    fn multiple_generic_multiple_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo<A, B> {
            bar: A,
            baz: B,
        }

        let expected = Foo::<String, &str> {
            bar: "Hello world!".to_string(),
            baz: "Hello world!",
        };
        let foo = Foo::<String, &str>::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!")
            .build();
        assert_eq!(expected, foo);

        // let foo: Foo<String, &str> = Foo::builder().bar("Hello world!".to_string()).baz("Hello world!").build();
        // assert_eq!(expected, foo);

        // let foo = Foo::builder().build();
    }

    #[test]
    fn multiple_generic_with_const_multiple_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo<A, B, const C: usize> {
            bar: A,
            baz: B,
        }

        let expected = Foo::<String, &str, 0> {
            bar: "Hello world!".to_string(),
            baz: "Hello world!",
        };
        let foo = Foo::<String, &str, 0>::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!")
            .build();
        assert_eq!(expected, foo);

        // let foo: Foo<String, &str> = Foo::builder().bar("Hello world!".to_string()).baz("Hello world!").build();
        // assert_eq!(expected, foo);

        // let foo = Foo::builder().build();
    }

    #[test]
    fn single_propagate() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            #[builder(propagate)]
            bar: Bar,
        }

        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Bar {
            baz: String,
        }

        let expected = Foo {
            bar: Bar {
                baz: "Hello world!".to_string(),
            },
        };
        let foo = Foo::builder()
            .bar(|builder| builder.baz("Hello world!".to_string()).build())
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().bar(|builder| builder.build() ).build();

        // #[derive(Debug, Default, PartialEq, Eq, Builder)]
        // pub struct Foo {
        //     #[builder(propagate)]
        //     bar: Bar,
        // }

        // #[derive(Debug, Default, PartialEq, Eq, Builder)]
        // pub struct Bar {
        //     baz: String,
        // }

        // let expected = Foo {
        //     bar: Bar {
        //         baz: "Hello world!".to_string(),
        //     }
        // };
        // let foo = Foo::builder().bar(|builder| builder.baz("Hello world!".to_string()).build() ).build();
        // assert_eq!(expected, foo);
    }

    #[test]
    fn optional_propagate() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Foo {
            #[builder(propagate)]
            bar: Option<Bar>,
        }

        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        pub struct Bar {
            baz: String,
        }

        let expected = Foo {
            bar: Some(Bar {
                baz: "Hello world!".to_string(),
            }),
        };
        let foo = Foo::builder()
            .bar(|builder| Some(builder.baz("Hello world!".to_string()).build()))
            .build();
        assert_eq!(expected, foo);

        // let foo = Foo::builder().bar(|builder| builder.build() ).build();

        // #[derive(Debug, Default, PartialEq, Eq, Builder)]
        // pub struct Foo {
        //     #[builder(propagate)]
        //     bar: Bar,
        // }

        // #[derive(Debug, Default, PartialEq, Eq, Builder)]
        // pub struct Bar {
        //     baz: String,
        // }

        // let expected = Foo {
        //     bar: Bar {
        //         baz: "Hello world!".to_string(),
        //     }
        // };
        // let foo = Foo::builder().bar(|builder| builder.baz("Hello world!".to_string()).build() ).build();
        // assert_eq!(expected, foo);
    }
}
