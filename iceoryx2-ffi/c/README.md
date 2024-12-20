# iceoryx2-ffi-C

## Build instructions - simple developer setup

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ffi/build
cmake --build target/ffi/build
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
cmake -S . -B target/ffi/build -DCMAKE_INSTALL_PREFIX=target/ffi/install -DBUILD_CXX_BINDING=OFF -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release"
cmake --build target/ffi/build
cmake --install target/ffi/build
```

The installed libraries can the be used for out-of-tree builds of the example or
custom C projects. This are the required steps:

```bash
cmake -S examples/c -B target/out-of-tree/examples/c -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/install"
cmake --build target/out-of-tree/examples/c
```
