# iceoryx2-ffi-cxx

## Build instructions

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ffi/build
cmake --build target/ffi/build
```

This is the most simple way to build the C++ bindings for `iceoryx2`, which rely
on the `iceorx_hoofs` C++ base library.

For production it is recommended though to separately build `iceoryx_hoofs` and
specify the path to the install directory with `-DCMAKE_PREFIX_PATH`.

`iceoryx_hoofs` can be build with this steps:

```bash
git clone https://github.com/eclipse-iceoryx/iceoryx.git target/iceoryx/src

cmake -S target/iceoryx/src/iceoryx_platform -B -DCMAKE_BUILD_TYPE=Release target/iceoryx/build/platform -DCMAKE_INSTALL_PREFIX=target/iceoryx/install
cmake --build target/iceoryx/build/platform
cmake --install target/iceoryx/build/platform

cmake -S target/iceoryx/src/iceoryx_hoofs -B target/iceoryx/build/hoofs -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=target/iceoryx/install -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install"
cmake --build target/iceoryx/build/hoofs
cmake --install target/iceoryx/build/hoofs
```

The C++ bindings can use the installed `iceoryx_hoofs` and be installed to be
used by custom projects. This are the steps:

```bash
cmake -S . -B target/ffi/build -DCMAKE_INSTALL_PREFIX=target/ffi/install -DCMAKE_PREFIX_PATH="$( pwd )/target/iceoryx/install"
cmake --build target/ffi/build
cmake --install target/ffi/build
```

The installed libraries can the be used for out-of-tree builds of the example or
custom C++ projects. This are the required steps:

```bash
cmake -S examples/cxx -B target/out-of-tree/examples/cxx -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/install;$( pwd )/target/iceoryx/install"
cmake --build target/out-of-tree/examples/cxx
```
