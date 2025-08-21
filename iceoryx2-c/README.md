<!-- markdownlint-disable-next-line MD044 -->
# iceoryx2-c

## Build instructions - simple developer setup

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_TESTING=ON
cmake --build target/ff/cc/build
```

This is the most simple way to build the C bindings for `iceoryx2`, which
utilizes cargo to build the Rust part of iceoryx2.

If only the C bindings should be build, without the C++ bindings, the
`-DBUILD_CXX=OFF` cmake parameter can be used.

## Build instructions for integrator

For production, it is recommended to separately build `iceoryx2-ffi-c`.

In the repository root folder, execute this steps:

```bash
cargo build --release --package iceoryx2-ffi-c

cmake -S iceoryx2-cmake-modules -B target/ff/cmake-modules/build
cmake --install target/ff/cmake-modules/build --prefix target/ff/cc/install

cmake -S iceoryx2-c -B target/ff/c/build \
      -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release" \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/c/build
cmake --install target/ff/c/build --prefix target/ff/cc/install
```

> [!NOTE]
> To pass `iceoryx2` feature flags to the `iceoryx2-ffi-c` crate, one needs to
> prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

The installed libraries can the be used for out-of-tree builds of the example or
custom C projects. This are the required steps:

```bash
cmake -S examples/c -B target/ff/out-of-tree/examples/c \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/out-of-tree/examples/c
```
