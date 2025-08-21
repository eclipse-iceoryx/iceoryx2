# iceoryx2-cxx

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

For production, it is recommended to separately build and install
`iceoryx2-ffi-c` and `iceoryx_hoofs`, and specify the path to the install
directory with `-DCMAKE_PREFIX_PATH`.

### Build and install `iceoryx2_ffi`

> [!NOTE]
> To pass `iceoryx2` feature flags to the `iceoryx2-ffi-c` crate, one needs to
> prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

First, build the C bindings generated from Rust:

```bash
cargo build --release --package iceoryx2-ffi-c
```

Then install the CMake package a discoverable location:

```bash
cmake -S iceoryx2-cmake-modules -B target/ff/cmake-modules/build
cmake --install target/ff/cmake-modules/build --prefix target/ff/cc/install

cmake -S iceoryx2-c -B target/ff/c/build \
      -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/release" \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/c/build
cmake --install target/ff/c/build --prefix target/ff/cc/install
```

### Build and install `iceoryx_hoofs`

Next, build the `iceoryx_platform` and `iceoryx_hoofs` libraries and install
the CMake packages at a discoverable location:

```bash
git clone --depth 1 --branch v2.95.7 https://github.com/eclipse-iceoryx/iceoryx.git target/ff/iceoryx/src

cmake -S target/ff/iceoryx/src/iceoryx_platform -B target/ff/iceoryx/build/platform \
      -DCMAKE_BUILD_TYPE=Release
cmake --build target/ff/iceoryx/build/platform
cmake --install target/ff/iceoryx/build/platform --prefix target/ff/cc/install

cmake -S target/ff/iceoryx/src/iceoryx_hoofs -B target/ff/iceoryx/build/hoofs \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install" \
      -DCMAKE_BUILD_TYPE=Release
cmake --build target/ff/iceoryx/build/hoofs
cmake --install target/ff/iceoryx/build/hoofs --prefix target/ff/cc/install
```

### Putting it together

The C++ bindings can then use the installed artifacts via
`-DCMAKE_PREFIX_PATH`. The C++ bindings can then be installed to be used by
custom projects.

```bash
cmake -S iceoryx2-cxx -B target/ff/cxx/build \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/cxx/build
cmake --install target/ff/cxx/build --prefix target/ff/cc/install
```

The installed libraries can the be used for out-of-tree builds of the example or
custom C++ projects. This are the required steps:

```bash
cmake -S examples/cxx -B target/ff/out-of-tree/examples/cxx \
      -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/out-of-tree/examples/cxx
```
