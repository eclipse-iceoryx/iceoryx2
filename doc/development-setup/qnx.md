# QNX Development Environment

> [!NOTE]
> The versions of Ubuntu supported by the QNX development platform can be
> found [here](
> https://support.qnx.com/developers/docs/relnotes8.0/com.qnx.doc.release_notes/topic/sdp8_rn.html)
>
> These instructions have been tested on Ubuntu 20.04.

## Installing

### Access the QNX toolchain

> [!NOTE]
> The QNX [non-commercial license](https://blackberry.qnx.com/en/products/qnx-everywhere/licensing)
> is available for open-source software development with QNX 8.0.
>
> A license is required for QNX 7.1.

Follow [these instructions](https://www.qnx.com/developers/docs/8.0/com.qnx.doc.qnxsdp.quickstart/topic/install_host.html)
to install the "QNX Software Center" for "Linux Hosts". Then, add your license
and install the "QNX Software Development Platform" via the
"QNX Software Center".

Following successful installation, a directory named either `qnx800` or
`qnx710` containing the QNX toolchain should be available in your `$HOME`
directory.

```bash
$ tree -L 2 $HOME/qnx710
qnx710
├── docs
│   └── enduser.pdf
├── host
│   ├── common
│   ├── linux
│   └── win64
├── qnxsdp-env.bat
├── qnxsdp-env.sh
└── target
    └── qnx7
```

## Emulating

### Create a QNX image for QEMU

> [!TIP]
> A convenience script is available for making QNX images for QEMU.
>
> See: `internal/scripts/qnx_make_qemu_image.sh --help`

The [`mkqnximage`](https://www.qnx.com/developers/docs/8.0/com.qnx.doc.neutrino.utilities/topic/m/mkqnximage.html)
CLI can be used to create a QNX image for development. See the `--help` for
available configuration options:

```bash
mkqnximage --help
```

The following image configuration was used for development of the platform
abstraction for QNX:

#### x86_64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export VM_HOSTNAME="x86_64-qnx-vm"
export VM_IPV4_ADDR="172.31.1.11"

export IMAGE_DIR="$HOME/images/minimal"
mkdir -p $IMAGE_DIR
cd $IMAGE_DIR

mkqnximage \
    --noprompt \
    --hostname="$VM_HOSTNAME" \
    --type=qemu \
    --arch=x86_64 \
    --ip="$VM_IPV4_ADDR" \
    --telnet=yes \
    --sys-size=256 \
    --sys-inodes=24000 \
    --data-size=256 \
    --data-inodes=24000
```

### Run a QNX image on QEMU

> [!TIP]
> A convenience script is available for running pre-built images.
>
> See: `internal/scripts/qnx_run_qemu_image.sh --help`

Images build with `mkqnximage` can be run using the `--run` option.

#### x86_64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export IMAGE_DIR="$HOME/images/minimal"
cd $IMAGE_DIR

mkqnximage --run
```

Alternatively, use QEMU directly for more fine-grained control over the
emulation:

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export IMAGE_DIR="$HOME/images/minimal"
export SHARED_DIR="${IMAGE_DIR}/shared"
mkdir -p $SHARED_DIR

export MAC=$(printf "52:54:00:%02x:%02x:%02x" $(( $RANDOM & 0xff)) $(( $RANDOM & 0xff )) $(( $RANDOM & 0xff)))

sudo ${QNX_TOOLCHAIN}/host/common/mkqnximage/qemu/net.sh /usr/lib/qemu/qemu-bridge-helper /etc/qemu/bridge.conf

qemu-system-x86_64 \
  -smp 2 \
  -m 1G \
  -drive file=${IMAGE_DIR}/output/disk-qemu.vmdk,if=ide,id=drv0 \
  -hdb fat:rw:${IMAGE_DIR}/shared \
  -netdev bridge,br=br0,id=net0 \
  -device e1000,netdev=net0,mac=$MAC \
  -nographic \
  -kernel ${IMAGE_DIR}/output/ifs.bin \
  -serial mon:stdio \
  -object rng-random,filename=/dev/urandom,id=rng0 \
  -device virtio-rng-pci,rng=rng0
```

### Connect to the QNX via telnet

`telnet` can be used to connect to a running image with default credentials
(root/root):

```bash
export VM_IPV4_ADDR="172.31.1.11" # Or whatever address you have chosen
telnet $VM_IPV4_ADDR
```

## Building

### Build the Rust toolchain for QNX

> [!TIP]
> A convenience script is available for building the Rust toolchain.
>
> See: `internal/scripts/qnx_build_rust_toolchain.sh --help`

In order to build Rust applications for QNX targets, a custom-built Rust
compiler is required due to the dependence on the QNX toolchain.

The QNX targets supported by the Rust compiler can be found in
[the `rustc` book](https://doc.rust-lang.org/rustc/platform-support/nto-qnx.html).

#### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source ${QNX_TOOLCHAIN}/qnxsdp-env.sh

# Clone Rust source
export RUSTDIR=~/source/rust
git clone https://github.com/rust-lang/rust.git -b 1.88.0 --depth 1 $RUSTDIR

# Configure the build
echo -e "[build]\nextended = true" > $RUSTDIR/config.toml

# Build the compiler (x86_64 and aarch64 targets)
cd $RUSTDIR

export build_env='
    CC_x86_64_pc_nto_qnx710=qcc
    CFLAGS_x86_64_pc_nto_qnx710=-Vgcc_ntox86_64_cxx
    CXX_x86_64_pc_nto_qnx710=qcc
    AR_x86_64_pc_nto_qnx710=ntox86_64-ar
    CC_aarch64_unknown_nto_qnx710=qcc
    CFLAGS_aarch64_unknown_nto_qnx710=-Vgcc_ntoaarch64le_cxx
    CXX_aarch64_unknown_nto_qnx710=qcc
    AR_aarch64_unknown_nto_qnx710=ntoaarch64-ar
    '
./x.py build --target aarch64-unknown-nto-qnx710,x86_64-pc-nto-qnx710,x86_64-unknown-linux-gnu rustc library/core library/alloc library/std library tools/rustfmt

# Create a symlink for easier use
export RUST_TOOLCHAIN="qnx-custom"
rustup toolchain link +${RUST_TOOLCHAIN} $RUSTDIR/build/host/stage1
```

### Build remote testing tools for QNX

> [!TIP]
> A convenience script is available for building the remote testing utilities.
>
> See: `internal/scripts/qnx_build_rust_toolchain.sh --help`

This is a TCP server that allows for transferring and running test binaries on
the QNX image on QEMU.

#### x86_64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source ${QNX_TOOLCHAIN}/qnxsdp-env.sh

export TOOLCHAIN_BIN_DIR="${QNX_TOOLCHAIN}/host/linux/x86_64/usr/bin"
export IMAGE_DIR="$HOME/images/default"
export SHARED_DIR="${IMAGE_DIR}/shared"
mkdir -p "$SHARED_DIR"

export RUSTDIR="$HOME/source/rust"
cd $RUSTDIR

export RUST_TOOLCHAIN="qnx-custom"

# Build the remote-test-client for the host and copy it into the QNX toolchain
cargo +${RUST_TOOLCHAIN} build --release --package remote-test-client
cp $RUSTDIR/target/release/remote-test-client $TOOLCHAIN_BIN_DIR/

# Build remote-test-server for the target and copy it into the mounted volume
cargo +${RUST_TOOLCHAIN} build --release --package remote-test-server --target x86_64-pc-nto-qnx710
cp $RUSTDIR/target/x86_64-pc-nto-qnx710/release/remote-test-server $SHARED_DIR/remote-test-server-x86_64
```

#### Aarch64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source ${QNX_TOOLCHAIN}/qnxsdp-env.sh

export TOOLCHAIN_BIN_DIR="${QNX_TOOLCHAIN}/host/linux/x86_64/usr/bin"
export IMAGE_DIR="$HOME/images/default"
export SHARED_DIR="${IMAGE_DIR}/shared"
mkdir -p "$SHARED_DIR"

export RUSTDIR="$HOME/source/rust"
cd $RUSTDIR

export RUST_TOOLCHAIN="qnx-custom"

# Build the remote-test-client for the host and copy it into the QNX toolchain
cargo +${RUST_TOOLCHAIN} build --release --package remote-test-client
cp $RUSTDIR/target/release/remote-test-client $TOOLCHAIN_BIN_DIR/

# Build remote-test-server for the target and copy it into the mounted volume
cargo +${RUST_TOOLCHAIN} build --release --package remote-test-server --target aarch64-unknown-nto-qnx710
cp $RUSTDIR/target/aarch64-unknown-nto-qnx710/release/remote-test-server $SHARED_DIR/remote-test-server-aarch64
```

### Build `iceoryx2` for QNX

Use the custom-built compiler to build for QNX targets:

#### x86_64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export RUST_TOOLCHAIN="qnx-custom"
cargo +${RUST_TOOLCHAIN} build --target x86_64-pc-nto-qnx710 --package iceoryx2
```

#### Aarch64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export RUST_TOOLCHAIN="qnx-custom"
cargo +${RUST_TOOLCHAIN} build --target aarch64-unknown-nto-qnx710 --package iceoryx2
```

## Testing

### Running the Test Suite

> [!TIP]
> A convenience script is available for building and running tests on a remote
> target.
> The `remote-test-server` will still need to be started on the target
> manually.
>
> See: `internal/scripts/remote_run_test_suite.sh --help`

#### x86_64

##### QNX 7.1

The tests can be built in a similar way to the library:

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export RUST_TOOLCHAIN="qnx-custom"
cargo +{$RUST_TOOLCHAIN} build --target x86_64-pc-nto-qnx710 --package iceoryx2 --tests
```

In order to execute the tests on the emulated target, first start the
`remote-test-server` from within the VM:

```bash
mount -t dos /dev/hd1t6 /mnt/shared
cp /mnt/shared/remote-test-server-x86_64 /data/home/root/remote-test-server
chmod +x /data/home/root/remote-test-server
RUST_TEST_THREADS=1 ./remote-test-server -v --bind 0.0.0.0:12345 --sequential
```

Then the tests can be executed using the `remote-test-client` from the host:

```bash
export VM_IPV4_ADDR="172.31.1.11"
export TEST_DEVICE_ADDR=$VM_IPV4_ADDR:12345

export RUSTDIR="$HOME/source/rust"
$RUSTDIR/build/host/stage0-tools-bin/remote-test-client run 0 <test_binary>
```

### Running Benchmarks

> [!TIP]
> A convenience script is available for building and running benchmarks on a
> remote target.
> The `remote-test-server` will still need to be started on the target
> manually.
>
> See: `internal/scripts/remote_run_benchmarks.sh --help`

#### x86_64

##### QNX 7.1

First build the benchmarks:

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source $QNX_TOOLCHAIN/qnxsdp-env.sh

export RUST_TOOLCHAIN="qnx-custom"
cargo +${RUST_TOOLCHAIN} build --release --target x86_64-pc-nto-qnx710 --package benchmark-publish-subscribe --package benchmark-event --package benchmark-request-response --package benchmark-queue
```

In order to execute the benchmarks on the emulated target, first start the
`remote-test-server` from within the VM:

```bash
mount -t dos /dev/hd1t6 /mnt/shared
cp /mnt/shared/remote-test-server-x86_64 /data/home/root/remote-test-server
chmod +x /data/home/root/remote-test-server
RUST_TEST_THREADS=1 /data/home/root/remote-test-server -v --bind 0.0.0.0:12345 --sequential
```

Then the benchmarks can be executed using the `remote-test-client` from the
host:

```bash
export VM_IPV4_ADDR="172.31.1.11"
export TEST_DEVICE_ADDR=$VM_IPV4_ADDR:12345

export RUSTDIR="$HOME/source/rust"
$RUSTDIR/build/host/stage0-tools-bin/remote-test-client run 0 <benchmark_binary> <benchmark_args>
```

## Debugging

### Remote Debugging with GDB

The GNU debugger `gdb` can be used to transfer binaries to QNX running on QEMU
and debug them.

First, start the [remote debug agent](https://www.qnx.com/developers/docs/8.0/com.qnx.doc.neutrino.user_guide/topic/security_pdebug.html?hl=pdebug)
in the QNX VM:

```sh
pdebug 1234
```

Then on the development host, connect to the target via `gdb`:

#### x86_64

##### QNX 7.1

```bash
export QNX_TOOLCHAIN="$HOME/qnx710"
source ${QNX_TOOLCHAIN}/qnxsdp-env.sh

ntox86_64-gdb
file path/to/binary
target qnx 172.31.1.11:1234 # If using same image as above
upload path/to/binary data/home/root/binary
run
```
