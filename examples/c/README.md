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

In the repository root folder, execute the following steps.

```bash
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```
