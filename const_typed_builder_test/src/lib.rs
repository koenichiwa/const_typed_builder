#[cfg(test)]
mod test {
    use const_typed_builder::Builder;

    #[test]
    fn compile_fail_tests() {
        let test_cases = trybuild::TestCases::new();
        test_cases.pass("./pass/empty_pass.rs");
        let test_dir = std::fs::read_dir("./compile_fail").expect("Can't find test directory");
        test_dir.for_each(|entry| {
            let entry = entry.expect("Can't find test entry");
            if entry.path().extension() == Some(std::ffi::OsStr::new("rs")) {
                test_cases.compile_fail(entry.path())
            }
        })
    }

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
    fn assume_mandatory() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[builder(assume_mandatory)]
        pub struct Foo {
            bar: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn assume_mandatory_explicit_optional() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[builder(assume_mandatory)]
        pub struct Foo {
            bar: Option<String>,
            baz: Option<String>,
            #[builder(optional)]
            quz: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
            baz: Some("Hello world!".to_string()),
            quz: None,
        };
        let foo = Foo::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn group() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(baz = single)]
        pub struct Foo {
            #[builder(group = baz)]
            bar: Option<String>,
            #[builder(group = baz)]
            baz: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
            baz: None,
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(expected, foo);
    }

    #[test]
    fn group_solver_compiler() {
        #[derive(Debug, Default, PartialEq, Eq, Builder)]
        #[group(baz = single)]
        #[builder(solver = compiler)]
        pub struct Foo {
            #[builder(group = baz)]
            bar: Option<String>,
            #[builder(group = baz)]
            baz: Option<String>,
        }

        let expected = Foo {
            bar: Some("Hello world!".to_string()),
            baz: None,
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

        let foo = Foo::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!".to_string())
            .build();

        assert_eq!(expected, foo);
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

        let foo: Foo<String, &str> = Foo::builder()
            .bar("Hello world!".to_string())
            .baz("Hello world!")
            .build();
        assert_eq!(expected, foo);
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
    }

    #[test]
    fn no_other_derive_necessary() {
        #[derive(Builder)]
        pub struct Foo {
            bar: String,
        }
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(foo.bar, "Hello world!");
    }

    #[test]
    fn skip_field() {
        #[derive(Debug, PartialEq, Builder)]
        pub struct Foo {
            bar: String,
            #[builder(skip)]
            baz: Option<String>,
        }
        let expected = Foo {
            bar: "Hello world!".to_string(),
            baz: None,
        };
        let foo = Foo::builder().bar("Hello world!".to_string()).build();
        assert_eq!(foo, expected);
    }
}
