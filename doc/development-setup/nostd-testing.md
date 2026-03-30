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
1. No manual boilerplate maintenance when adding tests
1. All tests must be executable with the standard test framework (`cargo test`)
1. All `no_std` tests must be executable with the custom test framework

## Design

### Definitions

1. **Test Harness:** Boilerplate for test entrypoints, test discovery
1. **Test Runner:** Test orchestration, result reporting
1. **Test Framework:** Combination of a test harness and test runner

NOTE: This terminology seems to be inconsistent in literature on this topic
      in the Rust ecosystem, sometimes even being used interchangeably.
      For the clarity in this document, the definitions above are used.

### Background

The following constraints were considered in the design of the solution:

1. Disabling the standard test harness (`harness = false`) must be done
   per-test-binary
1. The `#![feature(custom_test_frameworks)]`  mechanism for providing a custom
   test runner is only available on the `nightly` toolchain
1. The standard test framework does not provide the mechanism for testing a
   library with different feature flags enabled - only the default feature set
   is used, making it impossible to test a `no_std` configuration directly

### Structure

#### Common Library

To facilitate reuse of test case logic in both `std` and `no_std` contexts,
test cases are implemented and exported as functions in a common library.

The convention used for naming these libraries is: `${crate-name}-tests-common`

The common test library must expose an `std` feature, which must be forwarded
to `iceoryx2-bb-testing` to ensure compatibility with `no_std` testing:

```toml
[features]
default = []
std = [
  "my-crate/std",
  "iceoryx2-bb-testing/std",
  "iceoryx2-bb-testing-macros/std",
]
```

Test functions are annotated with the custom `#[test]` attribute to make them
discoverable by the custom test framework:

```rust
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn my_test_case() {
    // ... test logic
    assert_that!( /* .. */ )
}
```

When many tests share the same type list, annotate the containing module with
`#[tests(...)]`:

```rust
use iceoryx2_bb_testing_macros::tests;

#[tests(TypeA, TypeB)]
pub mod generic {
    #[test]
    pub fn my_generic_test<T>() {
        // ... test logic
    }

    #[test]
    pub fn another_generic_test<T>() {
        // ... test logic
    }
}
```

Test cases that rely on `std` functionality can be annotated so that they
are skipped in `no_std` contexts:

```rust
use iceoryx2_bb_testing_macros::requires_std;

#[test]
#[requires_std("panics")]
pub fn my_test_case_using_threads() {
    // ... test logic
}
```

The `#[should_panic]` attribute can be used and is handled in the `std` runner
(via `catch_unwind`) but ignored by the `no_std` runner. The `#[requires_std]`
annotation should be used in conjunction to skip panic tests in `no_std`
contexts.

```rust
#[test]
#[should_panic]
pub fn my_test_that_panics() {
    // ... test logic that is expected to panic
}
```

The test cases are organized into modules that mirror the modules in the
crate:

```console
my-crate-tests-common/
└── src/
    ├── lib.rs
    ├── module_a_tests.rs
    └── module_b_tests.rs
```

#### `std` Testing

A single `tests/main.rs` file that sets up the test harness is all that is
required. Additionally, the common library should be linked so that all
annotated test functions are registered for execution.

```console
my-crate/
├── src/
└── tests/
    └── main.rs
```

```rust
extern crate my_crate_tests_common;

iceoryx2_bb_testing::test_harness!();
```

The `Cargo.toml` for the crate must declare this test binary with
`harness = false` to disable the standard test harness:

```toml
[dev-dependencies]
my-crate-tests-common = { workspace = true, features = ["std"] }
iceoryx2-bb-testing = { workspace = true, features = ["std"] }

[[test]]
name = "main"
harness = false
```

Running tests works as usual with the custom test framework:

```console
cargo test --package my-crate
```

The tests can also be executed via the `just` script available in the
repository. This option only exists for consistency with running tests
using the custom framework.

```console
just test my_crate
```

Note that this approach produces single test binary to execute all tests.
To isolate each test in separate processes, `nextest` can be used:

```console
cargo nextest run --package my-crate
```

#### `no_std` Testing

For every crate that defines tests for `no_std` targets, an additional binary
crate is created with the naming convention: `${crate-name}-tests-nostd`.

Again, a single `main.rs` file is required to set up the test harness and
link to the common test library so that all annotated test functions
are registered for execution.

If the tests require heap allocation, a global allocator must also be defined
here:

```console
my-crate-tests-nostd/
└── src/
    └── main.rs
```

```rust
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate my_crate_tests_common;

// Required if tests use heap allocation:
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;

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

iceoryx2_bb_testing::test_harness!();
```

> [!NOTE]
> All `no_std` tests cannot be defined below the `iceoryx2-bb` architecture
> layer. To test components in lower levels (i.e. `iceoryx2-pal`), components
> must be re-exported at the `iceoryx2-bb` layer to be tested.

Although the crate is specifically for `no_std` testing, it must provide an
`std` feature enabled by default. This ensures the crate builds successfully
in `std` workspace builds, avoiding linker symbol clashes (e.g. the panic
handler):

```toml
[features]
# Default to noop to support std workspace builds
default = ["std"]

std = [
  "iceoryx2-bb-testing-macros/std",
  "iceoryx2-bb-testing/std"
]

[dependencies]
my-crate-tests-common = { workspace = true, features = [] }
iceoryx2-bb-testing = { workspace = true }
iceoryx2-bb-testing-macros = { workspace = true }
```

Tests using the `no_std` framework are executed via the binary produced
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

Additionally, all `no_std` tests in the workspace can also be executed with the
`just` script:

```console
just test workspace --no_std
```
