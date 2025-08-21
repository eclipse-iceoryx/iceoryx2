# Cross-compiling iceoryx2

## Cross-compile project with `cross`

Setup `cross` as described here: [cross-rs GitHub](https://github.com/cross-rs/cross).

Once `cross` is setup, navigate to root of the iceoryx2 repo and run
the command below.

```bash
cross build --target aarch64-unknown-linux-gnu --release --package iceoryx2-ffi-c
```

## Build C bindings

Run the following at the repo root:

```bash
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=OFF -DCMAKE_INSTALL_PREFIX=target/ff/cc/install -DBUILD_CXX=OFF -DRUST_BUILD_ARTIFACT_PATH="$( pwd )/target/aarch64-unknown-linux-gnu/release"
```

## Build C examples

Download the correct version of the ARM toolchain here:
[ARM toolchain downloads](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads).
Make sure to read the release notes and ensure it matches with the
glibc version of your target.

Extract the archive, make note of where it is.

Make a cmake file. You can put it anywhere. We'll just call it `cross-example.cmake`

### `cross-example.cmake`

```cmake
set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# absolute path to extracted ARM toolchain archive, it should look like this
set(TOOLCHAIN_ROOT "/full/path/to/.../arm-gnu-toolchain-12.2.rel1-x86_64-aarch64-none-linux-gnu")

set(CMAKE_C_COMPILER "${TOOLCHAIN_ROOT}/bin/aarch64-none-linux-gnu-gcc")
set(CMAKE_CXX_COMPILER "${TOOLCHAIN_ROOT}/bin/aarch64-none-linux-gnu-g++")

# libc and crt1.o live here
set(CMAKE_SYSROOT "${TOOLCHAIN_ROOT}/aarch64-none-linux-gnu/libc")

set(CMAKE_FIND_ROOT_PATH "${CMAKE_SYSROOT}")
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
```

Build using configuration in `cross-example.cmake`:

The following directories vary depending on your system:

`CMAKE_TOOLCHAIN_FILE` - Point to where `cross-example.cmake` is
located in your system

`CMAKE_PREFIX_PATH` - Point to where the `target/ff/cc/install` is
located within the cloned repo, just use an absolute path for your system

`iceoryx2-c_DIR` - Forces cmake to look for the iceoryx2-C headers

```bash
cmake -S examples/c/publish_subscribe \
  -B target/ff/out-of-tree/examples/c/publish_subscribe \
  -DCMAKE_TOOLCHAIN_FILE="/full/path/to/.../cross-example.cmake" \
  -DCMAKE_PREFIX_PATH="/full/path/to/.../iceoryx2/target/ff/cc/install" \
  -Diceoryx2-c_DIR="/full/path/to/.../iceoryx2/target/ff/cc/install/lib/cmake/iceoryx2-c" \
  -DCMAKE_FIND_DEBUG_MODE=ON
```

```bash
cmake --build target/ff/out-of-tree/examples/c/publish_subscribe
```

Your example binaries should be in `...target/ff/out-of-tree/examples/c/publish_subscribe`
