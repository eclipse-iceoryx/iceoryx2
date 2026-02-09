# `no_std` testing

The default Rust testing framework utilizes the `std` module for functionality
such as output, orchestration, panic catching, etc.

To enable the test suite to be executed on `no_std` targets, a custom test
framework is implemented. The approach specifically targets `no_std` POSIX
targets, such as QNX 8.0, however it is intended to be extended to all kinds
of `no_std` targets, including bare metal.

## Requirements

The following requirements steered the design of the solution:

1. Test logic must not be duplicated
1. All tests must be executable with the standard test framework
1. All `no_std` tests must be executable with the custom test framework

## Design

### Definitions

1. **Test Harness:** Boilerplate for test entrypoints, test discovery
1. **Test Runner:** Test orechestration, result reporting
1. **Test Framework:** Combination of a test harness and test runner

NOTE: This terminology seems to be inconsistent in literature on this topic
      in the Rust ecosystem, sometimes even being used interchangeably.
      For the clarity in this document, the definitions above are used.

### Background

Whilst Rust provides the mechanisms to both disable the standard test harness
for test binaries, and to specify custom test runners, these features have
limitations which make them not desirable for our use-case:

1. Disabling the standard test harness must be done per-test-binary making it
   clunky to simultaneously support the standard and a custom test framework
1. The mechanism for providing a custom test runner is only available on
   the `unstable` toolchain
1. The standard test framework does not provide the mechanism for testing a
   library with different feature flags enabled - only the default feature set
   is used
    * **NOTE:** This has been observed but seems to be an odd limitation.
            More research required.

### Structure

#### Common Library

To facilitate reuse of test case logic in the different contexts (standard
framework vs. custom framework), test cases are implemented and exported as
functions in a common library. The convention used for naming these libraries
is: `${crate-name}-tests-common`.

The common test library must exposed an `std` feature, which is forwarded
to `iceoryx2-bb-testing` to ensure compatibility with `no_std` testing:

```toml
[features]
default = []
std = [
  "my-crate/std",
  "iceoryx2-bb-testing/std",
]
```

All test cases must then use the `assert_that` macro for assertions, which is
compatible with `no_std` testing.

```rust
pub fn my_test_case() {
    // ... test logic
    assert_that!( /* .. */ )
}
```

Test cases that rely on `std` functionality can be annotated so that they
can be appropriately handled by the `no_std` framework, or skipped.
These annotations are ignored by the standard testing framework.

```rust
use iceoryx2_bb_testing_nostd_macros::requires_std;

#[requires_std("panics")]
pub fn my_test_case_that_panics() {
    // ... test logic
    assert_that!( /* .. */ )
}

#[requires_std("threading")]
pub fn my_test_case_that_panics() {
    // ... test logic
    assert_that!( /* .. */ )
}
```

The test cases are organized into modules that mirror the modules in the
crate:

```console
my-crate-tests-common/
└── src/
    ├── lib.rs
    ├── module_a.rs
    └── module_b.rs
```

#### Standard Test Framework

The convention for defining (integration) tests in the `tests` directory of the
crate is followed.

For every module in the common test library, a corresponding test binary that
uses the standard test framework is produced.
This is achieved by creating a file in the `tests` directory for each module
in the common test library.

```console
my-crate/
├── src/
└── tests/
    ├── module_a_tests.rs
    └── module_b_tests.rs
```

The tests defined by the standard test framework delegate to the common library
for the test logic.

```rust
use my_crate_tests_common::module_a::*;

#[test]
fn my_test_case() {
    module_a::my_test_case();
}
```

All annotations that the standard test framework works with can be
used as usual.

The common test library must be depended on with the `std` feature enabled to
ensure the tested crate and its tests use the `std` module instead of
`no_std` work-arounds:

```toml
[dev-dependencies]
my-crate-tests-common = { workspace = true, features = ["std"] }
```

#### Custom Test Framework

For every crate that defines tests for `no_std` targets, an additional binary
crate is created with the naming convention: `${crate-name}-tests-nostd`
The binary produced by this crate utilizes the `no_std` test harness and test
runner.

Like defining the tests for the standard test framework, a separate file is
created (but this time in the `src` directory) for each module in the common
test framework.

```console
my-crate-tests-nostd/
└── src/
    ├── main.rs
    ├── module_a_tests.rs
    └── module_b_tests.rs
```

Again like integration with the standard test framework, the test cases
defined for `no_std` delegate to the common library for the test logic.
A different annotation is used to integrate with the `no_std` test framework:

```rust
use iceoryx2_bb_testing_nostd_macros::inventory_test;
use my_crate_tests_common::module_a::*;

#[inventory_test]
fn my_test_case() {
    module_a::my_test_case();
}
```

A macro for defining generic tests for the `no_std` test framework is also
available.

```rust
#[inventory_test_generic(TypeA, TypeB)]
fn my_test_case() {
    module_a::my_test_case();
}
```

In the `main.rs`, all test modules are imported and the harness and runner is
bootstrapped via a provided convenience macro:

```rust

mod modula_a_tests;
mod module_b_tests;

iceoryx2_bb_testing_nostd::bootstrap!();
```

> [!NOTE]
> All `no_std` tests cannot be defined below the `iceoryx2-bb` architecture
> layer. To test components in lower levels (i.e. `iceoryx2-pal`, components
> must be re-exported at the `iceoryx2-bb` layer)

If the tests require the `alloc` crate, a global allocator must be defined:

```rust
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;

#[derive(Debug)]
pub struct GlobalHeapAllocator(HeapAllocator);

impl GlobalHeapAllocator {
    pub const fn new() -> Self {
        Self(HeapAllocator::new())
    }
}

unsafe impl GlobalAlloc for GlobalHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.0.allocate(layout) {
            Ok(ptr) => ptr.as_ptr() as *mut u8,
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(non_null) = NonNull::new(ptr) {
            self.0.deallocate(non_null, layout);
        }
    }
}

#[global_allocator]
static GLOBAL: GlobalHeapAllocator = GlobalHeapAllocator::new();
```

Although the crate is specifically for `no_std` testing, it must provide an
`std` feature, and it must be enabled by default. This feature must be
propagated to the `iceoryx2-bb-testing-nostd*` crates. This is to ensure that
the crate is still successfully built in `std` workspace builds. If not done,
the linker will complain about symbol clashes (e.g. the panic handler).

```toml
[features]
# Default to noop to support std workspace builds
default = ["std"]

std = [
  "iceoryx2-bb-testing-nostd-macros/std",
  "iceoryx2-bb-testing-nostd/std"
]
```

However, the crate must depend on the common test library with the `std`
feature disabled to ensure the tested crate and its tests do not use the
`std` module:

```toml
[dev-dependencies]
my-crate-tests-common = { workspace = true }
```

## Running

### Standard Framework

Tests using the standard framework are executed as usual with `cargo`:

```console
cargo test my_crate
```

The tests can also be executed via the `just` script available in the
repository. This option only exists for consistency with running tests
using the custom framework.

```console
just test my_crate
```

### Custom Framework

Tests using the custom `no_std` framework are executed via the binary produced
by the crate containing the `no_std` tests. In addition, the `core` and `alloc`
libraries must be built with `panic=abort`:

```console
RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C panic=abort" \
  cargo run \ 
  --package my-crate-tests-nostd \
  --no-default-features \
  -Zbuild-std=core,alloc 
```

The tests can also be executed via the `just` script available in the
repository. This is much more convenient for the custom test framework, given
the complexity of the command:

```console
just test my_crate --no_std
```

Additionally, all `no_std` tests can also be executed with the `just` script:

```console
just test workspace --no_std
```

Note that this will not execute all tests with the `no_std` framework, only
the tests specifically integrated as per the approach defined above.
