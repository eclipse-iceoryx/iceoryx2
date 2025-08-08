# iceoryx2-ffi-cxx

## Build instructions - simple developer setup

First, build the C bindings:

```bash
cmake -S iceoryx2-ffi/c -B target/ffi/c/build \
      -DCMAKE_INSTALL_PREFIX=target/ffi/c/install
cmake --build target/ffi/c/build
cmake --install target/ffi/c/build
```

Then, the C++ bindings can be built using the C bindings:

```bash
cmake -S iceoryx2-ffi/cxx -B target/ffi/cxx/build \
      -DCMAKE_PREFIX_PATH=$( pwd )/target/ffi/c/install \
      -DBUILD_EXAMPLES=ON -DBUILD_TESTING=ON
cmake --build target/ffi/cxx/build
```

This is the most simple way to build the C++ bindings for `iceoryx2`, which rely
on the `iceorx_hoofs` C++ base library and utilizes cargo to build the Rust part
of iceoryx2.

## Build instructions for integrator

For production, it is recommended to separately build and install
`iceoryx2-ffi` and `iceoryx_hoofs`, and specify the path to the install
directory with `-DCMAKE_PREFIX_PATH`.

### Build and install `iceoryx2_ffi`

> [!NOTE]
> To pass `iceoryx2` feature flags to the `iceoryx2-ffi` crate, one needs to
> prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

First, build the C bindings generated from Rust:

```bash
cargo build --release --package iceoryx2-ffi
```

Then install the CMake package a discoverable location:

```bash
cmake -S iceoryx2-ffi/c -B target/ffi/c/build \
      -DCMAKE_INSTALL_PREFIX=target/ffi/c/install \
      -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release"
cmake --build target/ffi/c/build
cmake --install target/ffi/c/build
```

### Build and install `iceoryx_hoofs`

Next, build the `iceoryx_platform` and `iceoryx_hoofs` libraries and install
the CMake packages at a discoverable location:

```bash
git clone --depth 1 --branch v2.95.6 https://github.com/eclipse-iceoryx/iceoryx.git target/iceoryx/src

cmake -S target/iceoryx/src/iceoryx_platform -B target/iceoryx/build/platform \
      -DCMAKE_INSTALL_PREFIX=target/iceoryx/install \
      -DCMAKE_BUILD_TYPE=Release
cmake --build target/iceoryx/build/platform
cmake --install target/iceoryx/build/platform

cmake -S target/iceoryx/src/iceoryx_hoofs -B target/iceoryx/build/hoofs \
      -DCMAKE_INSTALL_PREFIX=target/iceoryx/install \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install" \
      -DCMAKE_BUILD_TYPE=Release
cmake --build target/iceoryx/build/hoofs
cmake --install target/iceoryx/build/hoofs
```

### Putting it together

The C++ bindings can then use the existing Rust artifacts via
`-DRUST_BUILD_ARTIFACT_PATH` and the installed artifacts via
`-DCMAKE_PREFIX_PATH`. The C++ bindings can then be installed to be used by
custom projects.

```bash
cmake -S iceoryx2-ffi/cxx -B target/ffi/cxx/build \
      -DCMAKE_INSTALL_PREFIX=target/ffi/cxx/install \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install;$( pwd )/target/ffi/c/install"
cmake --build target/ffi/cxx/build
cmake --install target/ffi/cxx/build
```

The installed libraries can the be used for out-of-tree builds of the example or
custom C++ projects. This are the required steps:

```bash
cmake -S examples/cxx -B target/out-of-tree/examples/cxx \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install;$( pwd )/target/ffi/c/install;$( pwd )/target/ffi/cxx/install"
cmake --build target/out-of-tree/examples/cxx
```
