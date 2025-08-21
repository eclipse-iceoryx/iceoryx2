# Setup The Development Environment and Build iceoryx2 on Windows

## Windows Installation

In case you do not have a Windows installation, Microsoft provides free
developer images from
[here](https://developer.microsoft.com/en-us/windows/downloads/virtual-machines/).

> [!NOTE]
> Due to ongoing technical issues, as of October 23, 2024, downloads are
> temporarily unavailable.

Alternatively, [Quickemu](https://github.com/quickemu-project/quickemu) can be
used to
[download and install Windows](https://github.com/quickemu-project/quickemu/wiki/04-Create-Windows-virtual-machines).

> [!NOTE]
> You might need to obtain a license from Microsoft after installation.

## Development Setup

### Install Microsoft Build Tools

Since we are targeting native Windows builds, you need to install the
[Microsoft Build Tools](https://visualstudio.microsoft.com/en/visual-cpp-build-tools).

While the website mentions `Microsoft Build Tools for C++`, they are also
required for the Rust compiler on Windows.

During the installation process, you are presented with a list of options.
The most straightforward choice is `Desktop Development with C++`. Please select
that option and continue with the installation process.

### Install Rust

The easiest way to install Rust is via `Rustup` from
<!-- markdownlint-disable-next-line MD044 -->
[rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started).

Execute `rustup-init` and proceed with the standard installation.

### Install Git

There are multiple ways to install `git` on Windows. One option is
[gitforwindows.org](https://gitforwindows.org/). Another option is the
[chocolatey](https://community.chocolatey.org) community repository for Windows.

With `chocolatey`, you can follow the instructions for the
[individual install](https://chocolatey.org/install#individual) and then use the
following commands to install `git`.

```powershell
choco install -y git
```

Additional packages can be found [here](https://community.chocolatey.org/packages).

### Install CMake

To build the C and C++ bindings, `cmake` is required. Similar to `git`, there
are multiple options. You can get it from the
[CMake](https://cmake.org/download/) project itself or via `choco`. It is
suggested to to set the option to add `CMake` to the system `PATH` for all
users should be set when it is installed.

```powershell
choco install -y cmake --installargs 'ADD_CMAKE_TO_PATH=System'
```

### Install LLVM

In order to access some POSIX functions, `iceoryx2` uses `bindgen`, which in turn
needs `libclang`, which is provided by the LLVM project. We'll use `choco` to
get `llvm` with the following command.

```powershell
choco install -y llvm
```

### Install cargo-nextest [optional]

[cargo-nextest](https://nexte.st/) is a next-gen test runner for Rust. It can be
used as direct replacement for `cargo test` and has a nicer output than the
default test runner.

```powershell
cargo install cargo-nextest
```

## Build iceoryx2

### Get iceoryx2

> [!NOTE]
> If you installed `llvm`, `git` and `cmake` with `choco`, you need to open a
> new powershell instance to execute the following commands, else you will get
> a `the term 'git' is not recognized` error!

```powershell
git clone https://github.com/eclipse-iceoryx/iceoryx2.git
cd iceoryx2
```

### Build the Rust Crates

```powershell
cargo build --workspace --all-targets
```

### C and C++ Bindings

The simplest way to build the C and C++ bindings is with the following commands.

```powershell
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build -j 4
```

This will re-build the Rust crates and automatically download and build
`iceoryx_hoofs` for the C++ bindings. This is fine for a Development setup.
For a production setup, please have a look at `iceoryx2-c/README.md` and
`iceoryx2-cxx/README.md`. Although the instructions are for Linux, it
should be easy to adapt them for Windows.
