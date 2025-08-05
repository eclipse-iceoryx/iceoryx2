#!/bin/sh
# Copyright (c) 2024 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -e

RUST_TOOLCHAIN="stable"
RUST_BUILD_TYPE_FLAG=""
CMAKE_BUILD_TYPE_FLAG="-DCMAKE_BUILD_TYPE=Debug"

while (( "$#" )); do
  case "$1" in
    --mode)
        if [[ "$2" == "release" ]]; then
            RUST_BUILD_TYPE_FLAG="--release"
            CMAKE_BUILD_TYPE_FLAG="-DCMAKE_BUILD_TYPE=Release"
        fi
        shift 2
        ;;
    --toolchain)
        RUST_TOOLCHAIN="$2"
        shift 2
        ;;
    "help")
        echo "Build script for the 32-64 bit mixed mode PoC."
        echo ""
        echo "Options:"
        echo "    --mode                Specify the build type. Either 'release' or 'debug'"
        echo "    --toolchain           Specify the rust toolchain, e.g. 'stable' or 'beta'"
        echo "Args:"
        echo "    help                  Print this help"
        echo ""
        exit 0
        ;;
    *)
        echo "Invalid argument '$1'. Try 'help' for options."
        exit 1
        ;;
  esac
done

cd $(git rev-parse --show-toplevel)

for n in $(seq 15 2000)
do
    if [ -z $LIBCLANG_PATH ]
    then
        if ! [ -d /usr/local/llvm${n}/lib ]
        then
            break
        fi
    fi

    if [ -d /usr/local/llvm${n}/lib ]
    then
        export LIBCLANG_PATH=/usr/local/llvm${n}/lib/
    fi
done

echo "###################"
echo "# use libclang: ${LIBCLANG_PATH}"
echo "###################"
echo

export PATH=$PATH:$HOME/.cargo/bin
rustup default $RUST_TOOLCHAIN
# export RUSTFLAGS="-C debug-assertions"
cargo fmt --all -- --check
cargo clippy --workspace --exclude iceoryx2-tunnels-zenoh -- -D warnings

echo "###################"
echo "# Run cargo build #"
echo "###################"

cargo build --workspace --exclude iceoryx2-tunnels-zenoh --all-targets $RUST_BUILD_TYPE_FLAG

echo "######################"
echo "# Run cargo nextest #"
echo "#####################"

cargo nextest run --workspace --exclude iceoryx2-tunnels-zenoh --all-targets --no-fail-fast $RUST_BUILD_TYPE_FLAG

echo "###########################################################"
echo "# Clean the target directory to reduce memory usage on VM #"
echo "###########################################################"

cargo clean

# Skip everything from here onwards on FreeBSD due to limitations on the disk space when building on the VM
exit 0

echo "###########################"
echo "# Build language bindings #"
echo "###########################"

./internal/scripts/ci_build_and_install_iceoryx_hoofs.sh
rm -rf target/ff/iceoryx/build

# Build examples only in out-of-tree, else we are running out of disk space on the VM
cmake -S . -B target/ff/cc/build $CMAKE_BUILD_TYPE_FLAG -DBUILD_EXAMPLES=OFF -DBUILD_TESTING=ON -DCMAKE_INSTALL_PREFIX=target/ff/cc/install -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/iceoryx/install"
cmake --build target/ff/cc/build
cmake --install target/ff/cc/build

echo "##############################"
echo "# Run language binding tests #"
echo "##############################"

target/ff/cc/build/tests/iceoryx2-c-tests
target/ff/cc/build/tests/iceoryx2-cxx-tests

echo "################################################################"
echo "# Build language binding examples in out-of-tree configuration #"
echo "################################################################"

rm -rf target/ff/cc/build
cmake -S examples/c -B target/ff/out-of-tree-c $CMAKE_BUILD_TYPE_FLAG -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install"
cmake --build target/ff/out-of-tree-c
rm -rf target/ff/out-of-tree-c

cmake -S examples/cxx -B target/ff/out-of-tree-cxx $CMAKE_BUILD_TYPE_FLAG -DCMAKE_PREFIX_PATH="$( pwd )/target/ff/cc/install;$( pwd )/target/ff/iceoryx/install"
cmake --build target/ff/out-of-tree-cxx
rm -rf target/ff/out-of-tree-cxx
