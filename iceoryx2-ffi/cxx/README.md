## Build instructions

In the repository root folder, execute this steps.

```bash
cmake -S . -B target/ffi/build
cmake --build target/ffi/build
```

This is the most simple way to build the C++ bindings for `iceoryx2`, which rely on the `iceorx_hoofs` C++ base library.

For production it is recommended though to separately build `iceoryx_hoofs` and specify the path to the install directory
with `-DCMAKE_PREFIX_PATH`.

`iceoryx_hoofs` can be build with this steps:

```bash
git clone https://github.com/eclipse-iceoryx/iceoryx.git target/ffi/dep/iceoryx

cmake -S target/ffi/dep/iceoryx/iceoryx_platform -B target/ffi/dep/build/platform -DCMAKE_INSTALL_PREFIX=target/ffi/dep/install
cmake --build target/ffi/dep/build/platform
cmake --install target/ffi/dep/build/platform

cmake -S target/ffi/dep/iceoryx/iceoryx_hoofs -B target/ffi/dep/build/hoofs -DCMAKE_INSTALL_PREFIX=target/ffi/dep/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/dep/install"
cmake --build target/ffi/dep/build/hoofs
cmake --install target/ffi/dep/build/hoofs
```

The C++ bindings can use the installed `iceoryx_hoofs` and be installed to be used by custom projects. This are the steps:

```bash
cmake -S . -B target/ffi/build -DCMAKE_INSTALL_PREFIX=target/ffi/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/dep/install"
cmake --build target/ffi/build
cmake --install target/ffi/build
```

The installed libraries can the be used for out-of-tree builds of the example or custom C++ projects. This are the required steps:

```bash
cmake -S examples/cxx -B target/out-of-tree/examples/cxx -DCMAKE_PREFIX_PATH="$( pwd )/target/ffi/dep/install;$( pwd )/target/ffi/install"
cmake --build target/out-of-tree/examples/cxx
```
