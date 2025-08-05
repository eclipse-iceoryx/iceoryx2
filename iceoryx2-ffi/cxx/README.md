# iceoryx2-ffi-cxx

## Build instructions - simple developer setup

In the repository root folder, execute this steps:

```bash
cmake -S . -B target/ff/cc/build
cmake --build target/ff/cc/build
```

This is the most simple way to build the C++ bindings for `iceoryx2`, which rely
on the `iceorx_hoofs` C++ base library and utilizes cargo to build the Rust part
of iceoryx2.

Note that since the C++ bindings are implemented with the C bindings, both
libraries are built.

## Build instructions for integrator

For production, it is recommended to separately build `iceoryx2-ffi` and
`iceoryx_hoofs`.

### Use cargo to build `iceoryx-ffi`

In the repository root folder, execute this steps:

```bash
cargo build --release --package iceoryx2-ffi
```

> [!NOTE]
> To pass `iceoryx2` feature flags to the `iceoryx2-ffi` crate, one needs to
> prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

### Build and install `iceoryx_hoofs`

For production it is recommended though to separately build `iceoryx_hoofs` and
specify the path to the install directory with `-DCMAKE_PREFIX_PATH`.

`iceoryx_hoofs` can be build with this steps:

```bash
git clone --depth 1 --branch v2.95.6 https://github.com/eclipse-iceoryx/iceoryx.git target/ff/iceoryx/src

cmake -S target/ff/iceoryx/src/iceoryx_platform -B target/ff/iceoryx/build/platform -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install
cmake --build target/ff/iceoryx/build/platform
cmake --install target/ff/iceoryx/build/platform

cmake -S target/ff/iceoryx/src/iceoryx_hoofs -B target/ff/iceoryx/build/hoofs -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/ff/iceoryx/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/iceoryx/install"
cmake --build target/ff/iceoryx/build/hoofs
cmake --install target/ff/iceoryx/build/hoofs
```

### Putting it together

The C++ bindings can use the existing Rust artifacts via
`-DRUST_BUILD_ARTIFACT_PATH` and the installed `iceoryx_hoofs` via
`-DCMAKE_PREFIX_PATH`. The C++ bindings can be installed to be used by custom
projects. This are the steps:

```bash
cmake -S . -B target/ff/cc/build -DCMAKE_INSTALL_PREFIX=target/ff/cc/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/iceoryx/install" -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release"
cmake --build target/ff/cc/build
cmake --install target/ff/cc/build
```

The installed libraries can the be used for out-of-tree builds of the example or
custom C++ projects. This are the required steps:

```bash
cmake -S examples/cxx -B target/ff/out-of-tree/examples/cxx -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install;$( pwd )/target/ff/iceoryx/install"
cmake --build target/cc/out-of-tree/examples/cxx
```
