# `no_std` builds

Information on how `no_std` builds are organized in `iceoryx2`.

## Philosophy

All crates below `iceoryx2` in the layered architecture must be implemented
with `no_std` support as the first priority.
Support for `no_std` in crates above `iceoryx2` in the layered architecture is
optional.

## Optional use of `std`

If a crate is able to provide optimization of some form by leveraging
the `std` module, it may do so by defining the `std` feature. Usages of `std`
must only augment the base `no_std` implementation.

For all crates below `iceoryx2` in the layered architecture, the `std` feature
**must be disabled by default**.

Crates that never leverage the `std` module must be annotated as follows:

```rust
#![no_std]
```

Crates that optionally use the `std` module for optimization must be annotated
as follows:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
```

All usages of `std` must be gated behind the feature, for example:

```rust
#[cfg(not(feature = "std"))]
pub use iceoryx2_pal_concurrency_sync::once::Once;
#[cfg(feature = "std")]
pub use std::sync::Once;
```

## Propagation of the `std` feature

Any crate that has dependencies which have the `std` feature must either
propagate the feature to those crates, or explicitly omit it. There are
three scenarios:

### Crates that only build without `std`

These crates must explicitly set `features` on the dependency without including
`std`. This signals that `std` is intentionally not enabled, regardless of
whether other features are selected. For example:

```toml
iceoryx2-bb-loggers = { workspace = true, features = ["console"] }
iceoryx2-bb-posix = { workspace = true, features = [] }
```

### Crates that only build with `std`

Such crates may only exist above `iceoryx2` in the layered architecture. These
crates must explicitly enable the `std` feature.
For example:

```toml
iceoryx2-log = { workspace = true, features = ["std"] }
iceoryx2-services-discovery = { workspace = true, features = ["std"] }
iceoryx2 = { workspace = true, features = ["std"] }
iceoryx2-cal = { workspace = true, features = ["std"] }
iceoryx2-bb-loggers = { workspace = true, features = ["std", "console"]}
iceoryx2-bb-container = { workspace = true, features = ["std"] }
iceoryx2-bb-derive-macros = { workspace = true, features = ["std"] }
iceoryx2-bb-elementary = { workspace = true, features = ["std"] }
iceoryx2-bb-system-types = { workspace = true, features = ["std"] }
iceoryx2-bb-posix = { workspace = true, features = ["std"] }
```

### Crates that can build with or without `std`

These crates must conditionally propagate the feature to their dependencies.
For example:

```toml
[features]
default = []

std = [
  "iceoryx2-log/std",
  "iceoryx2-bb-concurrency/std",
  "iceoryx2-bb-container/std",
  "iceoryx2-bb-elementary/std",
  "iceoryx2-bb-derive-macros/std",
  "iceoryx2-bb-print/std",
  "iceoryx2-bb-system-types/std",
  "iceoryx2-pal-posix/std",
]
```

Note that even if the crate itself is purely `no_std`, it must still propagate
the `std` feature to its dependencies. The crate may be used in a build where
`std` is available, and in that case its dependencies should be able to
leverage it.

### Verifying `std` feature propagation

A verification script may be run to ensure the feature is properly omitted
or propagated to dependencies:

```console
just verify std-propagation iceoryx2    # specific crate
just verify std-propagation workspace   # entire workspace
```

## Building `iceoryx2`

The `iceoryx2` crate wires together the crates below it in the layered
architecture and propagates the `std` feature to them when needed. Therefore
switching between `std` and `no_std` builds only requires enabling or disabling
the feature on the `iceoryx2` crate.

### Cargo

```console
cargo build --package iceoryx2                                          # std build
cargo build --package iceoryx2 --no-default-features                    # no_std build
```

### CMake

```console
cmake -S . -B target/ff/cc/build                                        # std build
cmake --build target/ff/cc/build

cmake -S . -B target/ff/cc/build -DIOX2_FEATURE_STD=OFF                 # no_std build
cmake --build target/ff/cc/build
```

### Bazel

```console
USE_BAZEL_VERSION=7.x bazel build //iceoryx2/...                        # std build
USE_BAZEL_VERSION=7.x bazel build //iceoryx2/... --//:feature_std=off   # no_std build
```
