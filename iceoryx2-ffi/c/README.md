<!-- markdownlint-disable-next-line MD044 -->
# iceoryx2-ffi-c

## Build instructions - simple developer setup

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ffi/c-cxx/build
cmake --build target/ffi/c-cxx/build
```

This is the most simple way to build the C bindings for `iceoryx2`, which
utilizes cargo to build the Rust part of iceoryx2.

If only the C bindings should be build, without the C++ bindings, the
`-DBUILD_CXX_BINDING=OFF` cmake parameter can be used.

## Build instructions for integrator

For production, it is recommended to separately build `iceoryx2-ffi`.

In the repository root folder, execute this steps:

```bash
cargo build --release --package iceoryx2-ffi
cmake -S . -B target/ffi/c-cxx/build -DCMAKE_INSTALL_PREFIX=target/ffi/c-cxx/install -DBUILD_CXX_BINDING=OFF -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release"
cmake --build target/ffi/c-cxx/build
cmake --install target/ffi/c-cxx/build
```

> [!NOTE]
> To pass `iceoryx2` feature flags to the `iceoryx2-ffi` crate, one needs to
> prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

The installed libraries can the be used for out-of-tree builds of the example or
custom C projects. This are the required steps:

```bash
cmake -S examples/c -B target/out-of-tree/examples/c -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/c-cxx/install"
cmake --build target/out-of-tree/examples/c
```
