# How to Write Conformance Tests

This guide explains how to write and organize conformance tests using the
provided Rust macros and procedural attributes.

## 1. Overview

**Conformance tests** ensure that different implementations of a trait or
interface behave identically. This framework provides macros and attributes to
automate the generation of test modules and test cases for multiple System Under
Test (SUT) types.

## 2. Key Components

| Component | Description |
|-----------|-------------|
| `#[conformance_test]` | Marks a function as a conformance test. The function must be generic over the SUT type(s). |
| `#[conformance_test_module]` | Generates a declarative macro for a module, collecting all conformance tests and instantiating them for each SUT type. |
| `instantiate_conformance_tests!` | Instantiates the generated macro for a module and a list of SUT types. |

## 3. Writing Conformance Tests

### Step 1: Define a Test Module

Annotate your test module with `#[conformance_test_module]`:

```rs
#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod my_module {
    // ...
}
```

> [!NOTE]
> Due to the limitations of procedural macros, it's not possible to obtain the
> file name and, consequently, the module name. Therefore, the annotated test
> module name must match the module name determined by the file name. The parent
> module name is also required to reconstruct the fully qualified path in the
> generated declarative macro that will run the tests. Since clippy complains
> about repeating module names, this warning needs to be deactivated.

### Step 2: Write Test Functions

Write test functions and mark them with `#[conformance_test]`. These functions
must be generic over the SUT type(s):

```rs
#[allow(clippy::module_inception)]
#[conformance_test]
pub fn test_my_feature<T: MyTrait>() {
    // Test logic here
}
```

> [!NOTE]
> This is equivalent to a test written with the `generic_test` crate, except
> that the function must be public to use it in an external crate, and the
> `conformance_test` attribute must be used instead of the `test` attribute.

### Step 3: Instantiate Tests for SUT Types

Use the fully qualified path to the parent test module and
`instantiate_conformance_tests!` to run the tests for each SUT type.

Assuming the tests are part of a `my_test_crate` crate, which contains
the modules `my_module1` and `my_module2`, the tests would be instantiated as
follows:

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{SUT1, SUT2, SUT3};

mod sut1_impl {
    use super::*;
    instantiate_conformance_tests!(my_test_crate::my_module1, super::SUT1);
    instantiate_conformance_tests!(my_test_crate::my_module2, super::SUT1);
}

mod sut2_impl {
    use super::*;
    instantiate_conformance_tests!(my_test_crate::my_module1, super::SUT2);
    instantiate_conformance_tests!(my_test_crate::my_module2, super::SUT2);
}

mod sut3_impl {
    use super::*;
    instantiate_conformance_tests!(my_test_crate::my_module1, super::SUT3);
    instantiate_conformance_tests!(my_test_crate::my_module2, super::SUT3);
}
```

<!-- markdownlint-disable MD028 These are two distinct blockquote -->
> [!NOTE]
> The first parameter of `instantiate_conformance_tests!` is the fully qualified
> path to the parent test module, and all subsequent parameters are the generic
> parameters of the test functions, which must also be specified with the fully
> qualified path.

> [!NOTE]
> If the same conformance test module is instantiated multiple times, it needs
> to be wrapped in a separate module.
<!-- markdownlint-enable MD028 -->

## 4. Full Example

Suppose you have a trait `MyTrait` with the conformance tests in the
`my_test_crate` crate.

The `my_impl` crate has two implementations: `ImplA` and `ImplB`.

**Conformance Test Module in `my_test_crate`:**

```rs
#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod my_module {
    use super::*;

    #[conformance_test]
    pub fn test_feature_x<T: MyTrait>() {
        let sut = T::new();
        assert_eq!(sut.feature_x(), 42);
    }

    #[conformance_test]
    pub fn test_feature_y<T: MyTrait>() {
        let sut = T::new();
        assert!(sut.feature_y());
    }
}
```

**Instantiated Tests in `my_impl`:**

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{ImplA, ImplB};

mod impl_a {
    use super::*;
    instantiate_conformance_tests!(my_test_crate::my_module, super::ImplA);
}

mod impl_b {
    use super::*;
    instantiate_conformance_tests!(my_test_crate::my_module, super::ImplB);
}
```

This will generate and run `test_feature_x` and `test_feature_y` for both
`ImplA` and `ImplB`.

## 5. Pitfalls

Assuming the conformance test suit defines some types that need to be used in
the instantiation. In this case, it is recommended to define the types outside
of the conformance test module:

```rs
trait Foo {}
struc Bar {}
struc Baz {}

impl Foo for Bar {}
impl Foo for Baz {}

#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod my_module {
    use super::*;

    #[conformance_test]
    pub fn test_feature<T: MyTrait, U: Foo>() {
        // ...
    }
}
```

This prevents to duplicate the module name when the types are imported. To use
them in the macro, import them in the parent scope and use `super::`:

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{ImplA, ImplB};
use my_test_crate::my_module::{Bar, Foo};

mod impl_a {
    use super::*;

    mod bar {
        use super::*;
        instantiate_conformance_tests!(
            my_test_crate::my_module,
            super::ImplA,
            super::Bar
        );
    }

    mod baz {
        use super::*;
        instantiate_conformance_tests!(
            my_test_crate::my_module,
            super::ImplA,
            super::Baz
        );
    }
}

mod impl_b {
    use super::*;

    mod bar {
        use super::*;
        instantiate_conformance_tests!(
            my_test_crate::my_module,
            super::ImplB,
            super::Bar
        );
    }

    mod baz {
        use super::*;
        instantiate_conformance_tests!(
            my_test_crate::my_module,
            super::ImplB,
            super::Baz
        );
    }
}
```

## 6. How It Works

The `conformance_test_module` proc macro will parse the module for functions
with the `conformance_test` attribute and generates a declarative macro using
the specified conformance test module name as macro name. The declarative macro
will be accessible from the crate root.

We take advantage of the fact that in Rust, a macro can have the same name as
the root module of a crate, and `foo::module_name` and `foo::module_name!()` can
both exist at the same time.

Assuming we have this conformance test in the `my_module` module of
`my_module.rs` file in the `my_test_crate` crate:

```rs
#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod my_module {
    use super::*;

    #[conformance_test]
    pub fn test_feature_x<T: MyTrait>() {
        let sut = T::new();
        assert_eq!(sut.feature_x(), 42);
    }

    #[conformance_test]
    pub fn test_feature_y<T: MyTrait>() {
        let sut = T::new();
        assert!(sut.feature_y());
    }
}
```

The proc macro will then generate the declarative macro in the `my_module` module
with `my_module` as macro name:

```rs
#[macro_export]
macro_rules! my_module {
    ($module_path:path, $($sut_type:ty),+) => {
        mod my_module {
            use $module_path::*;
            #[test]
            fn test_feature_x() {
                my_module::test_feature_x::<$($sut_type),+>();
            }
            #[test]
            fn test_feature_y() {
                my_module::test_feature_y::<$($sut_type),+>();
            }
        }
    };
}
```

Assuming we have the following instantiation:

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{ImplA, ImplB};

mod impl_a {
    super::*;
    instantiate_conformance_tests!(my_test_crate::my_module, super::ImplA);
}

mod impl_b {
    super::*;
    instantiate_conformance_tests!(my_test_crate::my_module, super::ImplB);
}
```

The `instantiate_conformance_tests!` macro will then expand to this code:

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{ImplA, ImplB};

mod impl_a {
    super::*;
    my_test_crate::my_module!(my_test_crate::my_module, super::ImplA);
}

mod impl_b {
    super::*;
    my_test_crate::my_module!(my_test_crate::my_module, super::ImplB);
}
```

With the expansion of the generated `my_module!` declarative macro, we get the
following code:

```rs
use iceoryx2_bb_testing::instantiate_conformance_tests;
use my_impl::{ImplA, ImplB};

mod impl_a {
    super::*;
    mod my_module {
        use my_test_crate::my_module::*;
        #[test]
        fn test_feature_x() {
            my_module::test_feature_x::<super::ImplA>();
        }
        #[test]
        fn test_feature_y() {
            my_module::test_feature_y::<super::ImplA>();
        }
    }
}

mod impl_b {
    super::*;
    mod my_module {
        use my_test_crate::my_module::*;
        #[test]
        fn test_feature_x() {
            my_module::test_feature_x::<super::ImplB>();
        }
        #[test]
        fn test_feature_y() {
            my_module::test_feature_y::<super::ImplB>();
        }
    }
}
```
