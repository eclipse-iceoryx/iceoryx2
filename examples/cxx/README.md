# Installation Instructions

## Install dependencies

Since iceoryx2 is written in Rust we need to install that first. We recommend
the [official approach](https://www.rust-lang.org/tools/install).

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then C/C++ compiler and library dependencies must be installed.

### ArchLinux

```sh
sudo ./internal/scripts/install_dependencies_archlinux.sh
```

### Ubuntu

```sh
sudo ./internal/scripts/install_dependencies_ubuntu.sh
```

## Build

From the repository root folder, first build the C bindings:

```bash
cmake -S iceoryx2-ffi/c -B target/ffi/c/build  \
      -DCMAKE_INSTALL_PREFIX=target/ffi/c/install
cmake --build target/ffi/c/build
cmake --install target/ffi/c/build
```

Then, the C++ bindings and examples with:

```bash
cmake -S iceoryx2-ffi/cxx -B target/ffi/cxx/build \
      -DCMAKE_PREFIX_PATH=$( pwd )/target/ffi/c/install \
      -DBUILD_EXAMPLES=ON
cmake --build target/ffi/cxx/build
```
