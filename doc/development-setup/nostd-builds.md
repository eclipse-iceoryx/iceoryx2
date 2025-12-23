# `no_std` builds

Information on how `no_std` builds are organized in `iceoryx2`.

## Feature Flag

1. Crates that can provide some optimization or added functionality by
   leveraging the `std` module have the `std` feature defined
1. Crates that have the `std` feature defined may leverage `std`-only features
   of their dependent crates by enable corresponding features in those crates
1. Crates that have the `std` feature defined are `no_std` when the `std`
   feature is not enabled
1. Crates that can function optimally without the `std` module do **not** have
    the `std` feature defined
1. Crates that do not have the `std` feature defined are always `no_std`
1. All crates below the `iceoryx2` crate in the architecture have the `std`
   feature **disabled** by default
1. The `iceoryx2` crate has the `std` feature defined
1. The `iceoryx2` crate enables the `std` feature by default
1. When the `std` feature is enabled in `iceoryx2`, the feature is
   propagated down to the dependent lower-level crates that also define the
   `std` feature
1. Crates that depend on `iceoryx2` should specify explicitly whether they
   require the `std` feature enabled in `iceoryx2`

## Building

### Cargo

To build for `no_std` targets with Cargo, build the `iceoryx2` crate directly
with the `std` feature disabled by disabling the default features.

```console
cargo build --package iceoryx2 --no-default-features
```

### CMake

To build CMake projects (C/C++) for `no_std` targets with CMake, disable the
`std` feature via a CMake options

```console
cmake -S . -B target/ff/cc/build -DIOX2_FEATURE_STD=OFF
cmake --build target/ff/cc/build
```

### Bazel

To build for `no_std` targets with Bazel, disable every feature requiring
`std`:

```console
USE_BAZEL_VERSION=7.x bazel build //iceoryx2/... --//:feature_std=off
```

## Dependency

If developing a crate that depends on `iceoryx2`, explicitly specify whether to
use enable the `std` feature.

For crates intended for platforms with `std` support, enable the `std` feature
in the `Cargo.toml`:

```toml
iceoryx2 = { version = "x.y.z", default-features = false, features = ["std"] }
```

For crates intended for platform without `std` support, disable the `std`
feature in the `Cargo.toml`:

```toml
iceoryx2 = { version = "x.y.z", default-features = false }
```

In addition, the logger must be configured by enabling the appropriate feature
for your platform, for example:

```toml
iceoryx2=loggers = { version = "x.y.z", features = ["bare_metal"] }
```
